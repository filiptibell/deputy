use std::{path::Path, sync::Arc};

use tree_sitter::{Query, Tree};
use url::Url;

use super::language::TreeSitterLanguage;

/**
    A file with an associated tree-sitter language and tree.

    Used to parse and query the contents of a persistent file.
*/
#[derive(Debug, Clone)]
pub struct TreeSitterFile {
    pub(super) uri: Arc<Url>,
    pub(super) contents: Arc<str>,
    pub(super) language: TreeSitterLanguage,
    pub(super) tree: Tree,
}

impl TreeSitterFile {
    pub fn new(file_uri: impl Into<Arc<Url>>, contents: impl Into<Arc<str>>) -> Option<Self> {
        let uri: Arc<Url> = file_uri.into();
        let contents: Arc<str> = contents.into();

        let language = TreeSitterLanguage::from_file_uri(&uri)?;
        let tree = language
            .parser()
            .parse(contents.as_bytes(), None)
            .expect("no fallible flags set");

        Some(Self {
            uri,
            contents,
            language,
            tree,
        })
    }

    pub fn set_uri(&mut self, new_path: impl Into<Arc<Url>>) {
        let uri: Arc<Url> = new_path.into();
        self.uri = uri;
    }

    pub fn set_contents(&mut self, new_contents: impl Into<Arc<str>>) {
        let contents: Arc<str> = new_contents.into();
        self.contents = contents;
        self.tree = self
            .language
            .parser()
            .parse(self.contents.as_bytes(), None)
            .expect("no fallible flags set")
    }

    pub fn uri(&self) -> &Url {
        &self.uri
    }

    pub fn contents(&self) -> &str {
        &self.contents
    }

    pub fn query(&self, query: &str) -> Option<Query> {
        self.language.query(query)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        fn test(file_path: &str, contents: &str, language: Option<TreeSitterLanguage>) {
            let file_path = Path::new(file_path);
            let contents = contents.to_string();

            let file_uri = Url::from_file_path(file_path).unwrap();
            let file = TreeSitterFile::new(file_uri, contents);

            assert!(file.is_some() == language.is_some());
            assert!(file.is_none() || file.is_some_and(|f| f.language == language.unwrap()));
        }

        test("package.json", "{}", Some(TreeSitterLanguage::Json));
        test("Cargo.toml", "[header]", Some(TreeSitterLanguage::Toml));
        test("Cargo.lock", "[header]", Some(TreeSitterLanguage::Toml));
        test("wally.toml", "[header]", Some(TreeSitterLanguage::Toml));
        test("wally.lock", "[header]", Some(TreeSitterLanguage::Toml));
        test("rokit.toml", "[header]", Some(TreeSitterLanguage::Toml));

        test("package.txt", "{}", None);
        test("package.json.txt", "{}", None);
    }
}
