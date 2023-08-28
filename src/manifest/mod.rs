use std::ops::Range;

use itertools::Itertools;

use crate::toml::*;

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
                token_lb.kind.is_left_brace()
                    && token_key.kind.is_string()
                    && token_key.value.as_string() == "tools"
                    && token_rb.kind.is_right_brace()
            },
        );

        if let Some(section) = tools_section {
            let found_tools = tokens_no_comments
                .iter()
                .skip(section.2 .0 + 1)
                .step_by(3)
                .tuple_windows()
                .take_while(|(token_lhs, token_eq, token_rhs)| {
                    token_lhs.kind.is_string()
                        && token_eq.kind.is_equals()
                        && token_rhs.kind.is_string()
                })
                .collect::<Vec<_>>();

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
