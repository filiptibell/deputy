use async_language_server::{
    lsp_types::Position,
    server::Document,
    text_utils::RangeExt,
    tree_sitter::{Node as TsNode, Range as TsRange},
    tree_sitter_utils::{find_ancestor, find_child},
};

pub fn find_all_dependencies(doc: &Document) -> Vec<TsNode> {
    let Some(root) = doc.node_at_root() else {
        return Vec::new();
    };

    let mut cursor = root.walk();
    let mut deps = Vec::new();

    for top_level in root.children(&mut cursor) {
        let Some(key) = find_child(top_level, |c| c.kind() == "bare_key") else {
            continue;
        };

        if doc.node_text(key) != "tools" {
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

pub fn find_dependency_at(doc: &Document, pos: Position) -> Option<TsNode> {
    let node = doc.node_at_position(pos)?; // either the key or value
    let pair = find_ancestor(node, |a| a.kind() == "pair")?; // tool-name = "spec"

    let table = find_ancestor(node, |a| a.kind() == "table")?;
    let key = find_child(table, |c| c.kind() == "bare_key")?;
    if doc.node_text(key) != "tools" {
        return None;
    }

    Some(pair)
}

pub fn parse_dependency<'tree>(pair: TsNode<'tree>) -> Option<RokitDependency<'tree>> {
    Some(RokitDependency {
        alias: find_child(pair, |c| c.kind() == "bare_key")?,
        spec: find_child(pair, |c| c.kind() == "string")?,
    })
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct RokitDependency<'tree> {
    pub alias: TsNode<'tree>,
    pub spec: TsNode<'tree>,
}

impl RokitDependency<'_> {
    pub fn spec_ranges(&self, doc: &Document) -> RokitDependencySpecRanges {
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

        RokitDependencySpecRanges {
            owner,
            repository,
            version,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RokitDependencySpecRanges {
    pub owner: Option<TsRange>,
    pub repository: Option<TsRange>,
    pub version: Option<TsRange>,
}

impl RokitDependencySpecRanges {
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
