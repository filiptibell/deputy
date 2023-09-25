use std::{
    env::consts::{ARCH, OS},
    str::FromStr,
};

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Os {
    Windows,
    MacOS,
    Linux,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Arch {
    Aarch64,
    X86_64,
}

#[derive(Debug, Clone, Copy, Error)]
pub enum ArtifactCompatParseError {
    #[error("could not parse operating system from artifact name")]
    Os,
}

#[derive(Debug, Clone, Copy)]
pub struct ArtifactCompat {
    os: Os,
    arch: Arch,
}

impl ArtifactCompat {
    pub fn is_compatible(&self) -> bool {
        match (self.os, self.arch) {
            // NOTE: MacOS x86 compat on aarch64 assumes that the user has rosetta 2 installed
            (Os::Windows, Arch::X86_64) => OS == "windows" && ARCH == "x86_64",
            (Os::MacOS, Arch::Aarch64) => OS == "macos" && ARCH == "aarch64",
            (Os::MacOS, Arch::X86_64) => OS == "macos" && (ARCH == "aarch64" || ARCH == "x86_64"),
            (Os::Linux, Arch::X86_64) => OS == "linux" && ARCH == "x86_64",
            _ => false,
        }
    }
}

impl FromStr for ArtifactCompat {
    type Err = ArtifactCompatParseError;
    fn from_str(name: &str) -> Result<Self, Self::Err> {
        // Original name matching implementation is taken from Aftman with some additional tweaks, source here:
        // https://github.com/LPGhatguy/aftman/blob/412d5b5a54971a881bfd99ebd7976b48cf5dedfe/src/tool_source/mod.rs
        let name_low = name.to_ascii_lowercase();

        let os = if name_low.contains("windows")
            || name_low.contains("win32")
            || name_low.contains("win64")
        {
            Some(Os::Windows)
        } else if name_low.contains("macos")
            || name_low.contains("osx")
            || name_low.contains("darwin")
        {
            Some(Os::MacOS)
        } else if name_low.contains("linux") || name_low.contains("ubuntu") {
            Some(Os::Linux)
        } else {
            None
        };

        let arch = if name_low.contains("x86-64")
            || name_low.contains("x86_64")
            || name_low.contains("x64")
            || name_low.contains("amd64")
            || name_low.contains("win64")
        {
            Arch::X86_64
        } else if name_low.contains("aarch64") || name_low.contains("arm64") {
            Arch::Aarch64
        } else {
            // Default to x86 if no arch was found in the artifact name
            Arch::X86_64
        };

        Ok(Self {
            os: os.ok_or(ArtifactCompatParseError::Os)?,
            arch,
        })
    }
}
