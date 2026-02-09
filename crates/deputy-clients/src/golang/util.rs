/**
    Encodes a Go module path for use in proxy URLs.
    Uppercase letters become `!` + lowercase (e.g. `Azure` -> `!azure`).
*/
pub fn encode_module_path(module: &str) -> String {
    let mut result = String::with_capacity(module.len());
    for ch in module.chars() {
        if ch.is_ascii_uppercase() {
            result.push('!');
            result.push(ch.to_ascii_lowercase());
        } else {
            result.push(ch);
        }
    }
    result
}

/**
    Extracts GitHub owner and repository from a Go module path.
    Returns `None` for non-GitHub modules.

    - `github.com/owner/repo` -> `Some(("owner", "repo"))`
    - `github.com/owner/repo/v2` -> `Some(("owner", "repo"))`
    - `github.com/owner/repo/pkg/sub` -> `Some(("owner", "repo"))`
    - `golang.org/x/text` -> `None`
*/
pub fn extract_github_owner_repo(module: &str) -> Option<(String, String)> {
    let stripped = module.strip_prefix("github.com/")?;
    let mut parts = stripped.splitn(3, '/');
    let owner = parts.next()?;
    let repo = parts.next()?;
    if owner.is_empty() || repo.is_empty() {
        return None;
    }
    Some((owner.to_string(), repo.to_string()))
}
