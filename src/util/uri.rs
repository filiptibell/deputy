use std::ffi::OsStr;

use tower_lsp::lsp_types::Url;

pub trait LspUriExt
where
    Self: Sized,
{
    fn file_name(&self) -> Option<String>;
    fn with_file_name(&self, file_name: impl AsRef<OsStr>) -> Option<Self>;
}

impl LspUriExt for Url {
    fn file_name(&self) -> Option<String> {
        if let Ok(file_path) = self.to_file_path() {
            if let Some(file_name) = file_path.file_name() {
                return file_name.to_str().map(ToString::to_string);
            }
        }
        None
    }

    fn with_file_name(&self, file_name: impl AsRef<OsStr>) -> Option<Self> {
        if let Ok(file_path) = self.to_file_path() {
            let new_path = file_path.with_file_name(file_name);
            return Url::from_file_path(new_path).ok();
        }
        None
    }
}
