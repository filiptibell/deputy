use std::path::Path;

use tree_sitter::{Parser, Query};
use tree_sitter_language::LanguageFn;
use url::Url;

pub const JSON_FILE_NAMES: &[&str] = &["package.json"];
pub const TOML_FILE_NAMES: &[&str] = &["aftman.toml", "Cargo.toml", "rokit.toml", "wally.toml"];

/**
    Tree-sitter language for a given file.
*/
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeSitterLanguage {
    Json,
    Toml,
}

impl TreeSitterLanguage {
    pub fn from_file_extension(ext: &str) -> Option<Self> {
        match ext.trim().to_ascii_lowercase().as_ref() {
            "json" => Some(Self::Json),
            "toml" => Some(Self::Toml),
            _ => None,
        }
    }

    pub fn from_file_name(name: &Path) -> Option<Self> {
        let name = name.file_name()?;
        match name.to_str() {
            Some(n) if JSON_FILE_NAMES.contains(&n.trim()) => Some(Self::Json),
            Some(n) if TOML_FILE_NAMES.contains(&n.trim()) => Some(Self::Toml),
            _ => match Path::new(name).extension() {
                Some(ext) => Self::from_file_extension(ext.to_str()?),
                None => None,
            },
        }
    }

    pub fn from_file_uri(uri: &Url) -> Option<Self> {
        let path = Path::new(uri.path());
        Self::from_file_name(path)
    }

    pub fn language_fn(&self) -> LanguageFn {
        match self {
            Self::Json => tree_sitter_json::LANGUAGE,
            Self::Toml => tree_sitter_toml_ng::LANGUAGE,
        }
    }

    pub fn parser(&self) -> Parser {
        let f = self.language_fn();
        let mut parser = Parser::new();
        parser.set_language(&f.into()).unwrap();
        parser
    }

    pub fn query(&self, query: &str) -> Option<Query> {
        let f = self.language_fn();
        Query::new(&f.into(), query).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_file_extension() {
        fn test(ext: &str, language: Option<TreeSitterLanguage>) {
            assert_eq!(TreeSitterLanguage::from_file_extension(ext), language);
        }

        test("json", Some(TreeSitterLanguage::Json));
        test("toml", Some(TreeSitterLanguage::Toml));
        test("txt", None);
    }

    #[test]
    fn test_from_file_name() {
        fn test(name: &str, language: Option<TreeSitterLanguage>) {
            assert_eq!(
                TreeSitterLanguage::from_file_name(Path::new(name)),
                language
            );
        }

        test("package.json", Some(TreeSitterLanguage::Json));
        test("Cargo.toml", Some(TreeSitterLanguage::Toml));
        test("Cargo.lock", Some(TreeSitterLanguage::Toml));
        test("wally.toml", Some(TreeSitterLanguage::Toml));
        test("wally.lock", Some(TreeSitterLanguage::Toml));
        test("rokit.toml", Some(TreeSitterLanguage::Toml));

        test("package.toml", Some(TreeSitterLanguage::Toml));

        test("package.txt", None);
        test("package.json.txt", None);
    }
}
