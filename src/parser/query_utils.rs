use async_language_server::{
    lsp_types::{Position, Range},
    server::Document,
    tree_sitter::Node,
};

pub fn range_contains(range: Range, pos: Position) -> bool {
    range.start <= pos && pos <= range.end
}

pub fn range_extend(left: Range, right: Range) -> Range {
    Range {
        start: left.start.min(right.start),
        end: left.end.max(right.end),
    }
}

pub fn range_for_substring(original_range: Range, original_string: &str, substring: &str) -> Range {
    let offset = original_string.find(substring).unwrap() as u32;
    Range {
        start: Position {
            line: original_range.start.line,
            character: original_range.start.character + offset,
        },
        end: Position {
            line: original_range.start.line,
            character: original_range.start.character + offset + substring.len() as u32,
        },
    }
}

type NodeAndString<'a> = (Node<'a>, &'a str);

pub fn extract_key_child<'a>(doc: &'a Document, node: Node<'a>) -> Option<NodeAndString<'a>> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if matches!(child.kind(), "bare_key" | "quoted_key") {
            if let Some(rets) = extract_key(doc, child) {
                return Some(rets);
            }
        }
    }
    None
}

pub fn extract_key<'a>(doc: &'a Document, child: Node<'a>) -> Option<NodeAndString<'a>> {
    if child.kind() == "bare_key" {
        let text = doc.text().byte_slice(child.byte_range());
        if let Some(text) = text.as_str() {
            return Some((child, text));
        }
    } else if child.kind() == "quoted_key" {
        let quote_open = child.child(0).expect("valid quoted key");
        let quote_close = child.child(1).expect("valid quoted key");

        let pos_open = quote_open.byte_range().end;
        let pos_close = quote_close.byte_range().start;

        let text = doc.text().byte_slice(pos_open..pos_close);
        if let Some(text) = text.as_str() {
            return Some((child, text));
        }
    }
    None
}

pub fn extract_string_child<'a>(doc: &'a Document, node: Node<'a>) -> Option<NodeAndString<'a>> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if matches!(child.kind(), "string") {
            if let Some(rets) = extract_string(doc, child) {
                return Some(rets);
            }
        }
    }
    None
}

pub fn extract_string<'a>(doc: &'a Document, child: Node<'a>) -> Option<NodeAndString<'a>> {
    if child.kind() == "string" {
        let quote_open = child.child(0).expect("valid string");
        let quote_close = child.child(1).expect("valid string");

        let pos_open = quote_open.byte_range().end;
        let pos_close = quote_close.byte_range().start;

        let text = doc.text().byte_slice(pos_open..pos_close);
        if let Some(text) = text.as_str() {
            return Some((child, text));
        }
    }
    None
}

pub fn extract_key_value_pair<'a>(
    doc: &'a Document,
    child: Node<'a>,
) -> Option<(NodeAndString<'a>, NodeAndString<'a>)> {
    let key = extract_key_child(doc, child)?;
    let val = extract_string_child(doc, child)?;
    Some((key, val))
}
