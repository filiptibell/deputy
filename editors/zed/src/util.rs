use std::fs;

pub fn is_file(path: &str) -> bool {
    fs::metadata(path).map_or(false, |stat| stat.is_file())
}

pub fn is_dir(path: &str) -> bool {
    fs::metadata(path).map_or(false, |stat| stat.is_dir())
}
