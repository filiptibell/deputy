use tower_lsp::lsp_types::Url;

pub fn uri_to_file_name(uri: &Url) -> Option<String> {
    if let Ok(file_path) = uri.to_file_path() {
        if let Some(file_name) = file_path.file_name() {
            return file_name.to_str().map(ToString::to_string);
        }
    }
    None
}
