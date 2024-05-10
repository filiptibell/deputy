use std::io::{Error as IoError, ErrorKind as IoErrorKind, Result as IoResult};
use std::path::Path;

use tokio::process::Command;

const BUN_BINARY_NAME: &str = "bun";
const BUN_LOCKFILE_NAME: &str = "bun.lockb";

pub async fn convert_to_utf8(file_path: &Path, contents: &[u8]) -> IoResult<String> {
    convert_to_utf8_inner(file_path, contents)
        .await
        .map_err(|e| IoError::new(IoErrorKind::Other, e))
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

    match file_name.as_str() {
        BUN_LOCKFILE_NAME => match convert_bun_lockb(file_path).await {
            Ok(s) => Ok(s),
            Err(e) => Err(format!(
                "Failed to convert bun.lockb to utf8\
                \nFile path: {file_path:?}\
                \nFile name: {file_name:?}\
                \nError: {e}"
            )),
        },
        _ => match String::from_utf8(contents.to_vec()) {
            Ok(s) => Ok(s),
            Err(e) => Err(format!(
                "Failed to parse file as utf8\
                \nFile path: {file_path:?}\
                \nFile name: {file_name:?}\
                \nError: {e}"
            )),
        },
    }
}

async fn convert_bun_lockb(file_path: &Path) -> std::result::Result<String, String> {
    let bunpath = which::which(BUN_BINARY_NAME)
        .map_err(|e| format!("Failed to find path to bun executable: {e}"))?;
    let result = Command::new(bunpath)
        .arg(file_path.display().to_string())
        .output()
        .await
        .map_err(|e| format!("Failed to run bun on lockfile: {e}"))?;
    let stdout = String::from_utf8(result.stdout)
        .map_err(|e| format!("Failed to parse bun lockfile output as utf8: {e}"))?;
    Ok(stdout)
}
