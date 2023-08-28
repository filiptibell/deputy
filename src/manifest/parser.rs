use std::ops::Range;

use super::*;

pub(super) fn find_tools(
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
        return Err(ParserError::external("missing tools section"));
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
