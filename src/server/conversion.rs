use std::io::{Error as IoError, Result as IoResult};
use std::path::Path;

pub async fn convert_to_utf8(file_path: &Path, contents: &[u8]) -> IoResult<String> {
    convert_to_utf8_inner(file_path, contents)
        .await
        .map_err(IoError::other)
}

async fn convert_to_utf8_inner(file_path: &Path, contents: &[u8]) -> Result<String, String> {
    let file_name = file_path
        .file_name()
        .map(|f| f.to_os_string())
        .and_then(|f| f.to_str().map(|f| f.to_string()));

    let file_name = match file_name {
        Some(f) => f,
        None => return Err(format!("Path has no file name: {file_path:?}")),
    };

    match String::from_utf8(contents.to_vec()) {
        Ok(s) => Ok(s),
        Err(e) => Err(format!(
            "Failed to parse file as utf8\
            \nFile path: {file_path:?}\
            \nFile name: {file_name:?}\
            \nError: {e}"
        )),
    }
}
