use std::{
    collections::HashSet,
    error::Error,
    fmt::{self, Display},
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

use anyhow::{Context, Result};
use async_language_server::{
    lsp_types::{DiagnosticSeverity, NumberOrString, Position, Range, Url},
    oneshot::{WorkspaceDiagnosticConfig, WorkspaceDiagnosticReport, workspace_diagnostics},
};
use clap::Parser;
use miette::{GraphicalReportHandler, LabeledSpan, NamedSource, Severity, SourceCode, SourceSpan};

use crate::server::DeputyLanguageServer;

#[derive(Debug, Clone, Parser)]
pub struct AnalyzeCommand {
    #[arg(default_value = ".")]
    pub root: PathBuf,
    #[arg(long, env = "GITHUB_TOKEN")]
    pub github_token: Option<String>,
}

impl AnalyzeCommand {
    pub async fn run(self) -> Result<ExitCode> {
        let root = self
            .root
            .canonicalize()
            .with_context(|| format!("failed to resolve {}", self.root.display()))?;
        let server = DeputyLanguageServer::new();

        if let Some(github_token) = self.github_token {
            server.set_github_token(github_token);
        }

        let report = workspace_diagnostics(server, WorkspaceDiagnosticConfig::new(&root)).await?;
        let diagnostics = diagnostics(&root, &report);
        print_report(&diagnostics);

        Ok(if has_errors(&diagnostics) {
            ExitCode::FAILURE
        } else {
            ExitCode::SUCCESS
        })
    }
}

#[derive(Debug, Clone)]
struct AnalyzeDiagnostic {
    path: PathBuf,
    line: u32,
    column: u32,
    severity: Severity,
    message: String,
    help: Option<String>,
    code: Option<String>,
    source: NamedSource<String>,
    span: SourceSpan,
}

impl Display for AnalyzeDiagnostic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(formatter)
    }
}

impl Error for AnalyzeDiagnostic {}

impl miette::Diagnostic for AnalyzeDiagnostic {
    fn code<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.code
            .as_ref()
            .map(|code| Box::new(code) as Box<dyn Display>)
    }

    fn severity(&self) -> Option<Severity> {
        Some(self.severity)
    }

    fn help<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.help
            .as_ref()
            .map(|help| Box::new(help) as Box<dyn Display>)
    }

    fn source_code(&self) -> Option<&dyn SourceCode> {
        Some(&self.source)
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
        Some(Box::new(std::iter::once(
            LabeledSpan::new_primary_with_span(None, self.span),
        )))
    }
}

fn print_report(diagnostics: &[AnalyzeDiagnostic]) {
    if diagnostics.is_empty() {
        println!("No diagnostics");
        return;
    }

    println!("{}", summary(diagnostics));

    let handler = GraphicalReportHandler::new()
        .without_cause_chain()
        .with_context_lines(1)
        .with_links(false)
        .with_urls(false);

    for diagnostic in diagnostics {
        let mut rendered = String::new();
        handler
            .render_report(&mut rendered, diagnostic)
            .expect("rendering to a string cannot fail");
        println!("\n{rendered}");
    }
}

fn diagnostics(root: &Path, report: &WorkspaceDiagnosticReport) -> Vec<AnalyzeDiagnostic> {
    let mut diagnostics: Vec<_> = report
        .documents
        .iter()
        .flat_map(|document| {
            let path = document_path(&document.uri);
            let relative_path = relative_path(root, &document.uri);
            let source = path
                .as_ref()
                .and_then(|path| fs::read_to_string(path).ok())
                .unwrap_or_default();

            document.diagnostics().into_iter().map(move |diagnostic| {
                let (message, help) = split_message(&diagnostic.message);
                AnalyzeDiagnostic {
                    path: relative_path.clone(),
                    line: diagnostic.range.start.line + 1,
                    column: diagnostic.range.start.character + 1,
                    severity: severity(diagnostic.severity),
                    message,
                    help,
                    code: diagnostic.code.as_ref().map(diagnostic_code),
                    source: NamedSource::new(relative_path.display().to_string(), source.clone()),
                    span: source_span(&source, diagnostic.range),
                }
            })
        })
        .collect();

    diagnostics.sort_by(|a, b| {
        a.path
            .cmp(&b.path)
            .then_with(|| a.line.cmp(&b.line))
            .then_with(|| a.column.cmp(&b.column))
            .then_with(|| severity_rank(a.severity).cmp(&severity_rank(b.severity)))
    });
    diagnostics
}

fn has_errors(diagnostics: &[AnalyzeDiagnostic]) -> bool {
    diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == Severity::Error)
}

fn summary(diagnostics: &[AnalyzeDiagnostic]) -> String {
    let errors = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == Severity::Error)
        .count();
    let warnings = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == Severity::Warning)
        .count();
    let advice = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == Severity::Advice)
        .count();
    let files = diagnostics
        .iter()
        .map(|diagnostic| &diagnostic.path)
        .collect::<HashSet<_>>()
        .len();

    format!(
        "{} diagnostic{} in {} file{}: {errors} error{}, {warnings} warning{}, {advice} info",
        diagnostics.len(),
        if diagnostics.len() == 1 { "" } else { "s" },
        files,
        if files == 1 { "" } else { "s" },
        if errors == 1 { "" } else { "s" },
        if warnings == 1 { "" } else { "s" },
    )
}

fn document_path(uri: &Url) -> Option<PathBuf> {
    uri.to_file_path().ok()
}

fn relative_path(root: &Path, uri: &Url) -> PathBuf {
    uri.to_file_path()
        .ok()
        .and_then(|path| path.strip_prefix(root).ok().map(ToOwned::to_owned))
        .unwrap_or_else(|| uri.path().into())
}

fn split_message(message: &str) -> (String, Option<String>) {
    let mut lines = message.lines();
    let message = lines.next().unwrap_or_default().to_string();
    let help = lines.collect::<Vec<_>>().join("\n");

    (message, (!help.is_empty()).then_some(help))
}

fn severity(severity: Option<DiagnosticSeverity>) -> Severity {
    match severity {
        Some(DiagnosticSeverity::ERROR) => Severity::Error,
        Some(DiagnosticSeverity::WARNING) => Severity::Warning,
        _ => Severity::Advice,
    }
}

fn diagnostic_code(code: &NumberOrString) -> String {
    match code {
        NumberOrString::Number(code) => code.to_string(),
        NumberOrString::String(code) => code.clone(),
    }
}

fn source_span(source: &str, range: Range) -> SourceSpan {
    let start = byte_offset(source, range.start);
    let end = byte_offset(source, range.end)
        .max(start + 1)
        .min(source.len());

    (start, end - start).into()
}

fn byte_offset(source: &str, position: Position) -> usize {
    let mut offset = 0;
    let target_line = position.line as usize;

    for (line, text) in source.split_inclusive('\n').enumerate() {
        if line == target_line {
            let line_len = text.trim_end_matches(['\r', '\n']).len();
            return offset + (position.character as usize).min(line_len);
        }
        offset += text.len();
    }

    source.len()
}

fn severity_rank(severity: Severity) -> u8 {
    match severity {
        Severity::Error => 0,
        Severity::Warning => 1,
        Severity::Advice => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_diagnostic_help_from_message() {
        let (message, help) = split_message("No package exists\nDid you mean `serde`?");

        assert_eq!(message, "No package exists");
        assert_eq!(help.as_deref(), Some("Did you mean `serde`?"));
    }

    #[test]
    fn keeps_single_line_messages_without_help() {
        let (message, help) = split_message("No package exists");

        assert_eq!(message, "No package exists");
        assert_eq!(help, None);
    }

    #[test]
    fn converts_lsp_range_to_source_span() {
        let span = source_span(
            "serde = \"1.0\"\ntokio = \"1.0\"\n",
            Range {
                start: Position {
                    line: 1,
                    character: 8,
                },
                end: Position {
                    line: 1,
                    character: 13,
                },
            },
        );

        assert_eq!(span.offset(), 22);
        assert_eq!(span.len(), 5);
    }

    #[test]
    fn clamps_source_span_to_the_source() {
        let span = source_span(
            "serde = \"1.0\"",
            Range {
                start: Position {
                    line: 0,
                    character: 8,
                },
                end: Position {
                    line: 0,
                    character: 200,
                },
            },
        );

        assert_eq!(span.offset(), 8);
        assert_eq!(span.len(), 5);
    }

    #[test]
    fn maps_lsp_severity_to_miette_severity() {
        assert_eq!(severity(Some(DiagnosticSeverity::ERROR)), Severity::Error);
        assert_eq!(
            severity(Some(DiagnosticSeverity::WARNING)),
            Severity::Warning
        );
        assert_eq!(
            severity(Some(DiagnosticSeverity::INFORMATION)),
            Severity::Advice
        );
        assert_eq!(severity(None), Severity::Advice);
    }

    #[test]
    fn only_error_diagnostics_fail_analyze() {
        assert!(has_errors(&[diagnostic(Severity::Error)]));
        assert!(!has_errors(&[diagnostic(Severity::Warning)]));
        assert!(!has_errors(&[diagnostic(Severity::Advice)]));
    }

    fn diagnostic(severity: Severity) -> AnalyzeDiagnostic {
        AnalyzeDiagnostic {
            path: PathBuf::from("Cargo.toml"),
            line: 1,
            column: 1,
            severity,
            message: "message".to_string(),
            help: None,
            code: None,
            source: NamedSource::new("Cargo.toml", "message".to_string()),
            span: (0, 1).into(),
        }
    }
}
