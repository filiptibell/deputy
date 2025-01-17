use crate::util::is_file;
use std::fs;
use zed::http_client::{fetch, HttpMethod, HttpRequestBuilder};
use zed::Result;
use zed_extension_api::{self as zed};

const PERSONAL_ACCESS_TOKEN_FILE_PATH: &str = "github-personal-access-token";

pub fn validate(token: &str) -> Result<()> {
    // https://docs.github.com/en/rest/overview/authenticating-to-the-rest-api
    let request = HttpRequestBuilder::new()
        .url("https://api.github.com/octocat")
        .header("Authorization", format!("Bearer {token}"))
        .header("X-GitHub-Api-Version", "2022-11-28")
        .method(HttpMethod::Get)
        .build()
        .map_err(|e| format!("Failed to build http request: {e}"))?;
    fetch(&request).map_err(|e| {
        const STATUS_CODE_START: &str = "status code ";
        format!(
            "Token validation failed. {}",
            match e.find(STATUS_CODE_START) {
                Some(status_code_position) => {
                    let status_str = &e[status_code_position + STATUS_CODE_START.len()..];
                    match status_str {
                        "401 Unauthorized" => "Invalid token.".to_string(),
                        "403 Forbidden" => {
                            "Too many invalid attempts or token has insufficient permissions."
                                .to_string()
                        }
                        "404 Not Found" => "Token has insufficient permissions.".to_string(),
                        _ => format!("Http error: {}", &e),
                    }
                }
                None => format!("Http error: {}", &e),
            }
        )
    })?;
    Ok(())
}

pub fn run_set_personal_access_token(
    _command: zed::SlashCommand,
    args: Vec<String>,
) -> Result<zed::SlashCommandOutput, String> {
    if args.len() != 1 {
        return Err("Expected exactly one argument.".to_owned());
    }

    let auth_key = &args[0];
    validate(auth_key)?;
    fs::write(PERSONAL_ACCESS_TOKEN_FILE_PATH, auth_key)
        .map_err(|e| format!("Failed to write to GitHub personal access token file: {e}"))?;

    Ok(zed::SlashCommandOutput {
        text: "Successfully set the GitHub personal access token. Restart the language server for the changes to the effect.".to_owned(),
        sections: Vec::new(),
    })
}

pub fn run_remove_personal_access_token(
    _command: zed::SlashCommand,
    args: Vec<String>,
) -> Result<zed::SlashCommandOutput, String> {
    if args.len() != 0 {
        return Err("Expected no arguments.".to_owned());
    }

    if !is_file(PERSONAL_ACCESS_TOKEN_FILE_PATH) {
        return Err("No GitHub personal access token was set.".into());
    }
    fs::remove_file(PERSONAL_ACCESS_TOKEN_FILE_PATH)
        .map_err(|e| format!("Failed to remove personal access token file: {e}"))?;

    Ok(zed::SlashCommandOutput {
        text: "Successfully removed the GitHub personal access token if it was set. Restart the language server for the changes to take effect.".to_owned(),
        sections: Vec::new(),
    })
}

pub fn get_personal_access_token() -> Result<Option<String>> {
    if !is_file(PERSONAL_ACCESS_TOKEN_FILE_PATH) {
        return Ok(None);
    }
    let token = fs::read_to_string(PERSONAL_ACCESS_TOKEN_FILE_PATH)
        .map_err(|e| format!("Failed to read the GitHub personal access token file: {e}"))?;
    if token.is_empty() {
        return Ok(None);
    }
    Ok(Some(token))
}
