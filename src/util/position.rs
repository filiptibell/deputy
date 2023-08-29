#![allow(dead_code)]

use std::ops::Range;

use lsp_types::{Position, Range as LspRange};

pub fn position_to_offset(source: impl AsRef<str>, position: Position) -> usize {
    let source = source.as_ref();

    if position.line == 0 {
        return 1 + position.character as usize;
    }

    let last_newline_offset = source
        .char_indices()
        .filter(|&(_, c)| c == '\n')
        .nth((position.line - 1) as usize)
        .map(|(index, _)| index)
        .expect("Invalid position");

    let mut offset = 0;
    offset += last_newline_offset;
    offset += 1;
    offset += position.character as usize;
    offset
}

pub fn offset_to_position(source: impl AsRef<str>, offset: usize) -> Position {
    let source = source.as_ref();

    let mut newline_count = 0;
    let mut newline_last_idx = 0;
    for (index, char) in source.char_indices() {
        if index >= offset {
            break;
        }
        if char == '\n' {
            newline_count += 1;
            newline_last_idx = index;
        }
    }

    Position::new(newline_count, (offset - newline_last_idx - 1) as u32)
}

pub fn offset_range_to_range(source: impl AsRef<str>, range: Range<usize>) -> LspRange {
    let start = offset_to_position(source.as_ref(), range.start);
    let end = offset_to_position(source.as_ref(), range.end);
    LspRange::new(start, end)
}

pub fn range_to_offset_range(source: impl AsRef<str>, range: LspRange) -> Range<usize> {
    let start = position_to_offset(source.as_ref(), range.start);
    let end = position_to_offset(source.as_ref(), range.end);
    Range { start, end }
}
