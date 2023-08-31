use std::{ops::Range, str::FromStr};

use itertools::Itertools;

use crate::toml::*;

use super::tool_spec::*;

#[derive(Debug, Clone)]
pub struct ManifestToolsHeader {
    pub span: Range<usize>,
}

#[derive(Debug, Clone)]
pub struct ManifestToolsMap {
    pub tools: Vec<ManifestTool>,
}

#[derive(Debug, Clone)]
pub struct ManifestTool {
    pub key_span: Range<usize>,
    pub key_text: String,
    pub val_span: Range<usize>,
    pub val_text: String,
}

impl ManifestTool {
    pub fn spec(&self) -> Result<ToolSpec, ToolSpecError> {
        let len = self.val_text.len();
        if len == 2 {
            ToolSpec::from_str("")
        } else if self.val_text.starts_with('\"') || self.val_text.starts_with('\'') {
            ToolSpec::from_str(&self.val_text[1..len - 1])
        } else {
            panic!("Unknown string char")
        }
    }
}

#[derive(Debug, Clone)]
pub struct Manifest {
    pub source: String,
    pub tools_header: ManifestToolsHeader,
    pub tools_map: ManifestToolsMap,
}

impl Manifest {
    pub fn parse(source: impl Into<String>) -> ParserResult<Self> {
        let source = source.into();

        let tokens_no_comments = Token::parse_all(&source)?
            .into_iter()
            .filter(|t| !t.kind.is_comment())
            .collect::<Vec<_>>();

        let tools_section = tokens_no_comments.iter().enumerate().tuple_windows().find(
            |((_, token_lb), (_, token_key), (_, token_rb))| {
                token_lb.kind.is_left_bracket()
                    && token_key.kind.is_string()
                    && token_key.value.as_string() == "tools"
                    && token_rb.kind.is_right_bracket()
            },
        );

        if let Some(section) = tools_section {
            let mut found_tools = Vec::new();

            let start = section.2 .0;
            let end = tokens_no_comments.len() - 2;
            for index in start..end {
                let token_lhs = &tokens_no_comments[index];
                if !token_lhs.kind.is_string() {
                    continue;
                }
                let token_eq = &tokens_no_comments[index + 1];
                if !token_eq.kind.is_equals() {
                    continue;
                }
                let token_rhs = &tokens_no_comments[index + 2];
                if !token_rhs.kind.is_string() {
                    continue;
                }
                found_tools.push((token_lhs, token_eq, token_rhs))
            }

            Ok(Manifest {
                source,
                tools_header: ManifestToolsHeader {
                    span: Range {
                        start: section.0 .1.span.start,
                        end: section.2 .1.span.end,
                    },
                },
                tools_map: ManifestToolsMap {
                    tools: found_tools
                        .into_iter()
                        .map(|tool| ManifestTool {
                            key_span: tool.0.span.clone(),
                            key_text: tool.0.text.to_string(),
                            val_span: tool.2.span.clone(),
                            val_text: tool.2.text.to_string(),
                        })
                        .collect(),
                },
            })
        } else {
            Err(ParserError::external("missing 'tools' section"))
        }
    }
}
