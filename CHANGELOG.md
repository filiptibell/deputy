# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## `0.2.2` - January 9th, 2024

### Added

- Added support for cargo workspace dependencies in `Cargo.toml` files.

## `0.2.1` - December 31st, 2023

### Added

- Added a VSCode extension command to manually set a GitHub Personal Access Token.
  If you are using a private Wally registry and the index repository is not public, you will need to set this for the extension to work.

### Fixed

- Fixed crash when encountering empty TOML sections

## `0.2.0` - October 30th, 2023

### Added

- Added full support for NPM! This includes:

  - Autocomplete for package names and versions
  - Hover for info about a package (installed version, description, links)
  - Diagnostics for when a newer version is available + action to update

## `0.1.1` - October 24th, 2023

### Fixed

- Fixed invalid diagnostics for Wally dev dependencies

## `0.1.0` - September 26th, 2023

### Added

- Added diagnostics for unsupported operating system and/or architecture (Aftman)
- Added diagnostics for invalid dependency realms (Wally)

### Changed

- Improved consistency of diagnostic messages
- Documentation link to [docs.rs](https://docs.rs/) is now always included, even if crates don't have a documentation link in their metadata

### Fixed

- Fixed potential deadlock / hang for Wally diagnostics with many dependencies

## `0.0.4` - September 15th, 2023

### Fixed

- Fixed hovers sometimes not appearing and needing to re-hover over the same location

## `0.0.3` - September 14th, 2023

### Fixed

- Fixed language server executable not being bundled correctly on Windows
- Fixed `node_modules` being included in the packaged extension, size should be much smaller now

## `0.0.2` - September 14th, 2023

### Fixed

- Fixed crash for Wally manifests with empty dependency sections:

  ```toml
  # No longer crashes
  [dependencies]

  ```

## `0.0.1` - September 14th, 2023

Initial release
