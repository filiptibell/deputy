pub const DEFAULT_INDEX_URL: &str = "https://github.com/UpliftGames/wally-index";
pub const DEFAULT_FRONTEND_URL: &str = "https://wally.run";

pub fn is_default_index(index_url: &str) -> bool {
    let stripped_default = DEFAULT_INDEX_URL
        .trim_start_matches("https://github.com/")
        .trim_end_matches(".git")
        .to_ascii_lowercase();
    let stripped_index = index_url
        .trim_start_matches("https://github.com/")
        .trim_end_matches(".git")
        .to_ascii_lowercase();
    stripped_index == stripped_default
}

// FUTURE: If the frontend link ever gets added to the index config, we can use that
pub fn get_default_frontend_link(scope_and_name: &str, version: Option<&str>) -> String {
    let mut link = format!("{DEFAULT_FRONTEND_URL}/package/{scope_and_name}");
    if let Some(v) = version {
        link.push_str("?version=");
        link.push_str(v);
    }
    link
}
