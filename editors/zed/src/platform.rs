use zed_extension_api::{self as zed};

use crate::constants::{BINARY_NAME, BINARY_ROOT_DIR};

#[derive(Debug, Clone, Copy)]
pub struct PlatformDescriptor {
    os: &'static str,
    arch: &'static str,
    exe_suffix: &'static str,
}

impl PlatformDescriptor {
    pub fn current() -> Self {
        let (platform, arch) = zed::current_platform();

        let os = match platform {
            zed::Os::Windows => "windows",
            zed::Os::Linux => "linux",
            zed::Os::Mac => "macos",
        };

        let arch = match arch {
            zed::Architecture::Aarch64 => "aarch64",
            _ => "x86_64",
        };

        let exe_suffix = match platform {
            zed::Os::Windows => ".exe",
            zed::Os::Linux | zed::Os::Mac => "",
        };

        Self {
            os,
            arch,
            exe_suffix,
        }
    }

    pub fn release_asset_name(&self, version: &str) -> String {
        format!(
            "{BINARY_NAME}-{}-{}-{}.zip",
            version.trim_start_matches('v'),
            self.os,
            self.arch
        )
    }

    pub fn server_binary_root(&self) -> String {
        BINARY_ROOT_DIR.to_string()
    }

    pub fn server_binary_dir(&self, version: &str) -> String {
        format!("{BINARY_NAME}-{version}")
    }

    pub fn server_binary_path(&self) -> String {
        format!("{BINARY_NAME}{}", self.exe_suffix)
    }
}
