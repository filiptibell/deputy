use super::*;

#[derive(Debug, Clone, Copy)]
pub struct Wally;

#[tower_lsp::async_trait]
impl Tool for Wally {
    fn new() -> Self {
        Self
    }
}
