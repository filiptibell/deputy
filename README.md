<!-- markdownlint-disable MD033 -->
<!-- markdownlint-disable MD041 -->

<img align="right" width="256" src="assets/icon-256.png" alt="Tooling Language Server icon" />

<h1 align="center">Tooling Language Server</h1>

<div align="center">
  <a href="https://github.com/filiptibell/tooling-language-server/actions">
    <img src="https://shields.io/endpoint?url=https://badges.readysetplay.io/workflow/filiptibell/tooling-language-server/ci.yaml" alt="CI status" />
  </a>
  <a href="https://github.com/filiptibell/tooling-language-server/actions">
    <img src="https://shields.io/endpoint?url=https://badges.readysetplay.io/workflow/filiptibell/tooling-language-server/release.yaml" alt="Release status" />
  </a>
  <a href="https://github.com/filiptibell/tooling-language-server/blob/main/LICENSE.txt">
    <img src="https://img.shields.io/github/license/filiptibell/tooling-language-server.svg?label=License&color=informational" alt="Language server license" />
  </a>
</div>

<br/>

A language server supporting several tools and package managers:

- [Cargo](https://crates.io)
- [NPM](https://www.npmjs.com)
- [Rokit](https://github.com/rojo-rbx/rokit)
- [Wally](https://github.com/UpliftGames/wally)

Provides autocomplete, diagnostics for out-of-date versions, and more. <br/>
Check out the [features](#features) and [screenshots](#screenshots) sections for a full overview.

## Installation

The language server can be installed as an extension from:

- The [Visual Studio Marketplace](https://marketplace.visualstudio.com/items?itemName=filiptibell.tooling-language-server) (VSCode)
- The [Open VSX Registry](https://open-vsx.org/extension/filiptibell/tooling-language-server) (Cursor, Windsurf, ...)

## Features

- Autocomplete for names and versions
- Hover for information - includes description, links to documentation & more
- Diagnostics:
  - A newer version is available
  - The specified tool / package / version does not exist
- Quick actions on diagnostics - update to newest version

## Screenshots

### Hovers

<img src="assets/screenshots/cargo-screenshot-hover.png" alt="Hover" height="50%" width="50%" />

### Diagnostics

<img src="assets/screenshots/cargo-screenshot-diagnostics.png" alt="Diagnostics" height="50%" width="50%" />

### Completions

<img src="assets/screenshots/cargo-screenshot-completions.png" alt="Completions" height="50%" width="50%" />

## Development

The VSCode extension can be compiled and installed locally:

1. Clone the repository
2. Install [Just], [Rust], [VSCE] and the [VSCode CLI]
3. Run `just vscode-install` in the repository to install the extension

[Just]: https://github.com/casey/just
[Rust]: https://www.rust-lang.org/tools/install
[VSCE]: https://github.com/microsoft/vscode-vsce
[VSCode CLI]: https://code.visualstudio.com/docs/editor/command-line
