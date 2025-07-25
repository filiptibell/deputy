# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## `0.6.0` - June 22nd, 2025

**Tooling Language Server** is now **Deputy**!

The previous name was generic, verbose, and simply not fun enough.
It is still the same language server, albeit improved & with a fresh coat of paint.

### Added

- Added support for the [Zed](https://zed.dev) editor ([#8])
- Added support for NPM deprecations ([#18])

### Changed

- Rewrote the core language server using [async-language-server](https://github.com/filiptibell/async-language-server), leading to significant improvements in stability, fewer weird race conditions, and better handling of unicode in different editors. You may also notice performance improvements and less memory being used.
- The extension for VSCode now automatically downloads the latest version of the Deputy language server, instead of bundling it. This behavior can be disabled by simply adding the language server binary somewhere in your PATH.

### Removed

- Removed support for the Zap language from this extension - it did not really fit into this project, and is now a completely separate extension, which can be found [here](https://github.com/filiptibell/zap-language)

[#8]: https://github.com/filiptibell/deputy/pull/8
[#18]: https://github.com/filiptibell/deputy/pull/18

## `0.5.0` - March 12th, 2025

### Changed

- Improved diagnostics performance when using many [Cargo features](https://doc.rust-lang.org/cargo/reference/features.html)

### Fixed

- Fixed invalid diagnostics and missing completions for [Cargo optional dependencies](https://doc.rust-lang.org/cargo/reference/features.html#optional-dependencies)
- Fixed language server panicking and not recovering during certain completion requests

## `0.4.2` - February 6th, 2025

### Added

- Added support for diagnostics & completions in [Cargo features](https://doc.rust-lang.org/cargo/reference/features.html)
- Added support for aliased packages using `package` in [Cargo dependencies](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#renaming-dependencies-in-cargotoml)

### Fixed

- Fixed completions not showing up in empty version / feature strings
- Fixed yanked packages showing up in diagnostics when they shouldn't

## `0.4.1` - January 18th, 2025

### Added

- Added syntax highlighting and language support for [Zap](https://github.com/red-blox/zap)
- Added back support for [Aftman](https://github.com/LPGHatguy/aftman)
- Added the extension to the Open VSX Registry, for various editors
- Added an extension for the [Zed code editor](https://zed.dev/) (not yet published)

## `0.4.0` - January 7th, 2025

Version `0.4.0` refactors JSON and TOML parsing to use [tree-sitter](https://github.com/tree-sitter/tree-sitter) instead of a custom parser.
This means that the language server now handles errors & invalid syntax in both JSON and TOML much more gracefully.
More specifically, it will now provide most services (hover, completion, diagnostics) even if the file does not fully parse 🚀

### Added

- Added support for more kinds of links in hovers (Wally homepage and repository fields + more)
- Added support for merging of package scopes when using private Wally registries, matching Wally behavior
- Added support for completions even in files that would not normally parse, eg:

  ```toml
  # Cargo.toml

  [dependencies]
  tok|
  ```

  The language server will now complete "tokio" and various other packages at the `|` cursor!

### Changes

- Various performance improvements across the board for faster diagnostics and hover responsiveness
- Links, versions, and other information in hovers are now _much_ more consistent and look the same across Cargo, NPM, Rokit, and Wally

### Fixes

- Fixed missing hovers, completions, diagnostics, when using version specifiers and ranges like `"^x.y.z"`, `">=x.y.z"`, ...
- Fixed invalid diagnostics for newer versions being available when using `x.y.z-additional-info.1` and similar specifiers
- Fixed duplicate diagnostics and a couple other race conditions related to file opening
- Fixed invalid error diagnostics for NPM package names & others
- Other miscellaneous fixes, check GitHub issues and git history for more

### Breaking Changes

[Aftman](https://github.com/LPGhatguy/aftman) and [Foreman](https://github.com/Roblox/foreman) are no longer supported - due to the language server now being streamlined and providing more consistent features across all different tooling it supports.
Supporting both Aftman and Foreman would add a non-trivial amount of complexity with all of the changes in this version, and [Rokit](https://github.com/rojo-rbx/rokit) can replace both of them while keeping full compatibility.

## `0.3.0` - May 10th, 2024

### Added

- Added support for more js package managers (pnpm, yarn, bun)

### Changed

- Improved support for cargo workspaces
- Improved diagnostics responsiveness

### Fixed

- Fixed sometimes having to open a file twice to get diagnostics
- Fixed inaccurate / partial diagnostics when first opening a file

## `0.2.3` - April 24th, 2024

### Fixed

- Fixed valid version ranges such as `>=1.0.0, <2.0.0` being detected as invalid.

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
