use super::*;

#[derive(Debug, Clone, Copy)]
pub struct Aftman;

#[tower_lsp::async_trait]
impl Tool for Aftman {
    fn new() -> Self {
        Self
    }
}
