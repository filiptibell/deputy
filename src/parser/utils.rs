use async_language_server::{server::Document, tree_sitter::Node as TsNode};

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

pub fn table_key_parts(doc: &Document, node: TsNode) -> Vec<String> {
    let mut parts = Vec::new();
    if node.kind() == "table" {
        if let Some(key) = node.named_child(0) {
            if key.kind() == "bare_key" {
                // [dependencies]
                parts.push(doc.node_text(key).to_string());
            } else if key.kind() == "quoted_key" {
                // ["dependencies"]
                parts.push(unquote(doc.node_text(key)));
            } else if key.kind() == "dotted_key" {
                // [workspace.dependencies] etc
                let mut cursor = key.walk();
                for child in key.children(&mut cursor) {
                    if child.kind() == "bare_key" {
                        parts.push(doc.node_text(child).to_string());
                    } else if child.kind() == "quoted_key" {
                        parts.push(unquote(doc.node_text(child)));
                    }
                }
            }
        }
    }
    parts
}
