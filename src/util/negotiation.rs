use lsp_types::{InitializeParams, PositionEncodingKind};

use tracing::info;

pub fn negotiate_position_encoding(params: &InitializeParams) -> PositionEncodingKind {
    let encoding_capabilities = match &params.capabilities.general {
        Some(general) => general.position_encodings.as_ref(),
        None => None,
    };
    match encoding_capabilities {
        Some(encodings) => {
            let mut encodings_str = String::new();
            for (index, encoding) in encodings.iter().enumerate() {
                encodings_str.push('"');
                encodings_str.push_str(encoding.as_str());
                encodings_str.push('"');
                if index < (encodings.len() - 1) {
                    encodings_str.push_str(", ");
                }
            }
            info!("Client supports position encodings {encodings_str}");
            if encodings.contains(&PositionEncodingKind::UTF8) {
                PositionEncodingKind::UTF8
            } else {
                PositionEncodingKind::UTF16
            }
        }
        _ => {
            info!("Client supports position encodings N/A");
            PositionEncodingKind::UTF16
        }
    }
}

pub fn negotiate_offset_encoding(params: &InitializeParams) -> String {
    let offset_encoding = match &params.capabilities.offset_encoding {
        Some(encodings) => {
            let mut encodings_str = String::new();
            for (index, encoding) in encodings.iter().enumerate() {
                encodings_str.push('"');
                encodings_str.push_str(encoding.as_str());
                encodings_str.push('"');
                if index < (encodings.len() - 1) {
                    encodings_str.push_str(", ");
                }
            }
            info!("Client supports offset encodings {encodings_str}");
            if encodings.contains(&"utf-8".to_string()) || encodings.contains(&"utf8".to_string()) {
                "utf-8"
            } else {
                "utf-16"
            }
        }
        _ => {
            info!("Client supports offset encodings N/A");
            "utf-16"
        }
    };
    offset_encoding.to_string()
}
