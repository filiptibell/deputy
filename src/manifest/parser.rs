use std::ops::Range;

use logos::Lexer;
use thiserror::Error;

use super::token::*;
use super::*;

#[derive(Debug, Error)]
pub enum ParserError {
    #[error("Failed to parse manifest")]
    ParsingFailed,
    #[error("Failed to parse manifest - incomplete input")]
    ParsingIncomplete,
    #[error("Failed to parse manifest - missing 'tools' section")]
    MissingTools,
}

#[derive(Debug, Clone)]
pub struct ParsedToken {
    pub token: Token,
    pub span: Range<usize>,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct ParsedTokens {
    tokens: Vec<ParsedToken>,
}

impl ParsedTokens {
    pub fn new(source: impl AsRef<str>) -> Result<Self, ParserError> {
        let mut lex = Lexer::<Token>::new(source.as_ref());

        let mut tokens = Vec::new();
        while let Some(res) = lex.next() {
            match res {
                Err(()) => return Err(ParserError::ParsingFailed),
                Ok(tok) => tokens.push(ParsedToken {
                    token: tok,
                    span: lex.span(),
                    text: lex.slice().to_string(),
                }),
            }
        }

        Ok(Self { tokens })
    }

    fn find_token_sequence(
        &self,
        offset: usize,
        seq: impl AsRef<[Token]>,
    ) -> Option<(Range<usize>, &[ParsedToken])> {
        let len = self.tokens.len();
        let seq = seq.as_ref();

        // Skip if either parsed tokens or given sequence is empty
        if len == 0 || seq.is_empty() {
            return None;
        }

        // Skip completely if seq length goes beyond parsed tokens length
        let end = len - seq.len();
        if offset > end {
            return None;
        }

        let first_tok = &seq[0];
        for index in offset..=end {
            let parsed = &self.tokens[index];
            if &parsed.token == first_tok {
                let mut matched_all = true;
                for index_inner in index..index + seq.len() {
                    let current_tok = &seq[index_inner - index];
                    let parsed_tok = &self.tokens[index_inner].token;
                    if parsed_tok != current_tok {
                        matched_all = false;
                        break;
                    }
                }
                if matched_all {
                    let range = Range {
                        start: index,
                        end: index + seq.len(),
                    };
                    let tokens = &self.tokens[range.clone()];
                    return Some((range, tokens));
                }
            }
        }

        None
    }
}

pub fn find_tools(
    parsed_tokens: &ParsedTokens,
) -> Result<(ManifestToolsHeader, ManifestToolsMap), ParserError> {
    let mut header_index_offset = 0;
    let mut header = None;
    while let Some((range, sequence)) = parsed_tokens.find_token_sequence(
        header_index_offset,
        [Token::LeftBracket, Token::Key, Token::RightBracket],
    ) {
        header_index_offset = range.end;
        if sequence[1].text == "tools" {
            header = Some(ManifestToolsHeader {
                span: Range {
                    start: sequence[0].span.start,
                    end: sequence[2].span.end,
                },
            });
            break;
        }
    }

    if header.is_none() {
        return Err(ParserError::MissingTools);
    }

    let mut tools_index_offset = header_index_offset;
    let mut tools = Vec::new();
    while let Some((range, sequence)) = parsed_tokens.find_token_sequence(
        tools_index_offset,
        [Token::Key, Token::Assignment, Token::String],
    ) {
        tools_index_offset = range.end;
        tools.push(ManifestTool {
            key_span: sequence[0].span.clone(),
            key_text: sequence[0].text.to_string(),
            val_span: sequence[2].span.clone(),
            val_text: sequence[2].text.to_string(),
        });
    }

    let map = ManifestToolsMap { tools };
    Ok((header.unwrap(), map))
}
