# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## `0.1.1` - October 24th, 2023

### Fixed

- Fixed invalid diagnostics for wally dev dependencies

## `0.1.0` - September 26th, 2023

### Added

- Added diagnostics for unsupported operating system and/or architecture (aftman)
- Added diagnostics for invalid dependency realms (wally)

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

- Fixed crash for wally manifests with empty dependency sections:

  ```toml
  # No longer crashes
  [dependencies]

  ```

## `0.0.1` - September 14th, 2023

Initial release
