use async_language_server::{server::Document, tree_sitter::Node as TsNode};

#[must_use]
pub fn unquote(text: impl AsRef<str>) -> String {
    let text = text.as_ref();
    if (text.starts_with('\'') && text.ends_with('\''))
        || (text.starts_with('"') && text.ends_with('"'))
    {
        text[1..text.len() - 1].to_string()
    } else {
        text.to_string()
    }
}

#[must_use]
pub fn key_part_nodes(node: TsNode<'_>) -> Vec<TsNode<'_>> {
    if matches!(node.kind(), "bare_key" | "quoted_key") {
        vec![node]
    } else if node.kind() == "dotted_key" {
        let mut cursor = node.walk();
        node.children(&mut cursor)
            .filter(|child| matches!(child.kind(), "bare_key" | "quoted_key"))
            .collect()
    } else {
        Vec::new()
    }
}

#[must_use]
pub fn key_parts(doc: &Document, node: TsNode) -> Vec<String> {
    key_parts_with(node, &|part| doc.node_text(part))
}

#[must_use]
pub fn key_parts_with<F>(node: TsNode, node_text: &F) -> Vec<String>
where
    F: Fn(TsNode) -> String,
{
    key_part_nodes(node)
        .into_iter()
        .map(|part| {
            if part.kind() == "quoted_key" {
                unquote(node_text(part))
            } else {
                node_text(part)
            }
        })
        .collect()
}

#[must_use]
pub fn table_key_parts(doc: &Document, node: TsNode) -> Vec<String> {
    if node.kind() == "table"
        && let Some(key) = node.named_child(0)
    {
        key_parts(doc, key)
    } else {
        Vec::new()
    }
}
