use async_language_server::lsp_types::{Position, Range};

pub fn range_contains(range: Range, pos: Position) -> bool {
    range.start <= pos && pos <= range.end
}
