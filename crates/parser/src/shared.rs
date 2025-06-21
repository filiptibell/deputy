use async_language_server::{
    lsp_types::Position,
    server::Document,
    text_utils::RangeExt,
    tree_sitter::{Node as TsNode, Range as TsRange},
    tree_sitter_utils::{find_ancestor, find_child},
};

#[derive(Debug, Clone, Copy)]
pub enum TableNames {
    Rokit,
    Wally,
}

fn check_table_name(table_names: TableNames, key: &str) -> bool {
    match table_names {
        TableNames::Rokit => key == "tools",
        TableNames::Wally => {
            ["dependencies", "dev-dependencies", "server-dependencies"].contains(&key)
        }
    }
}

pub(super) fn find_all_dependencies(doc: &Document, table_names: TableNames) -> Vec<TsNode> {
    let Some(root) = doc.node_at_root() else {
        return Vec::new();
    };

    let mut cursor = root.walk();
    let mut deps = Vec::new();

    for top_level in root.children(&mut cursor) {
        let Some(key) = find_child(top_level, |c| c.kind() == "bare_key") else {
            continue;
        };

        if !check_table_name(table_names, doc.node_text(key).as_str()) {
            continue;
        }

        let mut top_level_cursor = top_level.walk();
        for child in top_level.children(&mut top_level_cursor) {
            if child.kind() == "pair" {
                deps.push(child);
            }
        }
    }

    deps
}

pub(super) fn find_dependency_at(
    doc: &Document,
    pos: Position,
    table_names: TableNames,
) -> Option<TsNode> {
    let node = doc.node_at_position(pos)?; // either the key or value
    let pair = find_ancestor(node, |a| a.kind() == "pair")?; // tool-name = "spec"

    let table = find_ancestor(node, |a| a.kind() == "table")?;
    let key = find_child(table, |c| c.kind() == "bare_key")?;
    if !check_table_name(table_names, doc.node_text(key).as_str()) {
        return None;
    }

    Some(pair)
}

#[must_use]
pub fn parse_dependency(pair: TsNode) -> Option<TriDependency> {
    Some(TriDependency {
        alias: find_child(pair, |c| c.kind() == "bare_key")?,
        spec: find_child(pair, |c| c.kind() == "string")?,
    })
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct TriDependency<'tree> {
    pub alias: TsNode<'tree>,
    pub spec: TsNode<'tree>,
}

impl TriDependency<'_> {
    #[must_use]
    pub fn spec_ranges(&self, doc: &Document) -> TriDependencySpecRanges {
        let mut text = doc.node_text(self.spec);
        let mut range = self.spec.range();

        if (text.starts_with('\'') && text.ends_with('\''))
            || (text.starts_with('"') && text.ends_with('"'))
        {
            text.pop();
            text = text[1..].to_string();
            range.start_byte += 1;
            range.start_point.column += 1;
            range.end_byte -= 1;
            range.end_point.column -= 1;
        }

        let (owner, repository, version) = range.sub_delimited_tri(text.as_str(), '/', '@');

        TriDependencySpecRanges {
            owner,
            repository,
            version,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TriDependencySpecRanges {
    pub owner: Option<TsRange>,
    pub repository: Option<TsRange>,
    pub version: Option<TsRange>,
}

impl TriDependencySpecRanges {
    #[must_use]
    pub fn text<'a>(
        &self,
        doc: &'a Document,
    ) -> (Option<&'a str>, Option<&'a str>, Option<&'a str>) {
        let txt = doc.text();

        let owner = self
            .owner
            .as_ref()
            .map(|r| r.start_byte..r.end_byte)
            .and_then(|r| txt.byte_slice(r.clone()).as_str());
        let repository = self
            .repository
            .as_ref()
            .map(|r| r.start_byte..r.end_byte)
            .and_then(|r| txt.byte_slice(r.clone()).as_str());
        let version = self
            .version
            .as_ref()
            .map(|r| r.start_byte..r.end_byte)
            .and_then(|r| txt.byte_slice(r.clone()).as_str());

        (owner, repository, version)
    }
}
