# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

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
