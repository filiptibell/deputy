use std::fs;
use util::{is_dir, is_file};
use zed::LanguageServerId;
use zed_extension_api::{self as zed, Result};

mod github_auth;
mod util;

struct DeputyExtension {
    cached_binary_path: Option<String>,
}

impl DeputyExtension {
    fn language_server_binary_path(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<String> {
        if let Some(path) = worktree.which("deputy") {
            return Ok(path);
        }

        if let Some(path) = &self.cached_binary_path {
            if is_file(path) {
                return Ok(path.clone());
            }
        }

        zed::set_language_server_installation_status(
            &language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );

        let release = zed::latest_github_release(
            "filiptibell/deputy",
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;
        let (platform, arch) = zed::current_platform();
        let version = {
            let mut chars = release.version.chars();
            chars.next();
            chars.as_str()
        };
        let asset_name = format!(
            "deputy-{}-{}-{}.zip",
            version,
            match platform {
                zed::Os::Mac => "macos",
                zed::Os::Windows => "windows",
                zed::Os::Linux => "linux",
            },
            match arch {
                zed::Architecture::Aarch64 => "aarch64",
                _ => "x86_64",
            },
        );
        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == asset_name)
            .ok_or_else(|| format!("no asset found matching {:?}", asset_name))?;

        const BINARY_DIR: &str = "binary";
        let version_dir_name = format!("deputy-{}", release.version);
        let version_dir = format!("{BINARY_DIR}/{version_dir_name}");
        let binary_path = format!(
            "{version_dir}/deputy{extension}",
            extension = match platform {
                zed::Os::Mac | zed::Os::Linux => "",
                zed::Os::Windows => ".exe",
            }
        );

        if !is_dir(BINARY_DIR) {
            fs::create_dir(BINARY_DIR)
                .map_err(|e| format!("failed to create directory for binary: {e}"))?;
        }

        if !is_file(&binary_path) {
            zed::set_language_server_installation_status(
                &language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );

            zed::download_file(
                &asset.download_url,
                &version_dir,
                zed::DownloadedFileType::Zip,
            )
            .map_err(|e| format!("failed to download file: {e}"))?;

            zed::make_file_executable(&binary_path)?;

            let entries = fs::read_dir(BINARY_DIR)
                .map_err(|e| format!("failed to list working directory {e}"))?;
            for entry in entries {
                let entry = entry.map_err(|e| format!("failed to load directory entry {e}"))?;
                if entry.file_name().to_str() != Some(&version_dir_name) {
                    fs::remove_dir_all(&entry.path()).ok();
                }
            }
        }

        self.cached_binary_path = Some(binary_path.clone());

        Ok(binary_path)
    }
}

impl zed::Extension for DeputyExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let mut env = Vec::new();

        let github_auth_token = github_auth::get_personal_access_token()?;
        if let Some(token) = github_auth_token {
            env.push(("GITHUB_TOKEN".to_string(), token));
        }

        Ok(zed::Command {
            command: self.language_server_binary_path(language_server_id, worktree)?,
            args: vec!["serve".into()],
            env,
        })
    }

    fn run_slash_command(
        &self,
        command: zed::SlashCommand,
        args: Vec<String>,
        _worktree: Option<&zed::Worktree>,
    ) -> Result<zed::SlashCommandOutput, String> {
        match command.name.as_str() {
            "deputy-set-github-pat" => github_auth::run_set_personal_access_token(command, args),
            "deputy-remove-github-pat" => {
                github_auth::run_remove_personal_access_token(command, args)
            }
            command => Err(format!("unknown slash command: \"{command}\"")),
        }
    }
}

zed::register_extension!(DeputyExtension);
