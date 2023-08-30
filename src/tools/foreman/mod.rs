use super::*;

#[derive(Debug, Clone, Copy)]
pub struct Foreman;

#[tower_lsp::async_trait]
impl Tool for Foreman {
    fn new() -> Self {
        Self
    }
}
