use std::ops::Range;

use logos::Lexer;

use super::error::*;
use super::token::*;

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
    pub fn new(source: impl AsRef<str>) -> ParserResult<Self> {
        let mut lex = Lexer::<Token>::new(source.as_ref());

        let mut tokens = Vec::new();
        while let Some(res) = lex.next() {
            match res {
                Err(()) => return Err(ParserError::parsing_failed("N/A")),
                Ok(tok) => tokens.push(ParsedToken {
                    token: tok,
                    span: lex.span(),
                    text: lex.slice().to_string(),
                }),
            }
        }

        Ok(Self { tokens })
    }

    pub fn find_token_sequence(
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
