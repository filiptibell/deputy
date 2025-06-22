use std::{fs, io};

use zed_extension_api::Result;

pub fn file_exists(path: &str) -> bool {
    fs::metadata(path).is_ok_and(|meta| meta.is_file())
}

pub fn create_dir_if_nonexistent(path: &str) -> Result<()> {
    match fs::create_dir(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => Ok(()),
        Err(e) => Err(format!("failed to create directory at \"{path}\": {e}")),
    }
}

pub fn cleanup_dir_entries(path: &str, excluded: &str) -> Result<()> {
    let reader = fs::read_dir(path)
        .map_err(|e| format!("failed to cleanup directory at \"{path}\": {e}"))?;

    for entry in reader {
        let entry = entry.map_err(|e| format!("failed to cleanup directory at \"{path}\": {e}"))?;
        if entry.file_name() != excluded {
            fs::remove_dir_all(entry.path()).ok();
        }
    }

    Ok(())
}
