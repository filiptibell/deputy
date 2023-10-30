<!-- markdownlint-disable MD033 -->
<!-- markdownlint-disable MD041 -->

<img align="right" width="256" src="assets/icon-256.png" />

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

- [Aftman](https://github.com/LPGhatguy/aftman) / [Foreman](https://github.com/roblox/foreman)
- [Cargo](https://crates.io)
- [NPM](https://www.npmjs.com)
- [Wally](https://github.com/UpliftGames/wally)

Provides autocomplete, diagnostics for out-of-date versions, and more. <br/>
Check out the [features](#features) section for a full list of features.

## Installation

The language server can be installed as an extension from:

- The [Visual Studio Marketplace](https://marketplace.visualstudio.com/items?itemName=filiptibell.tooling-language-server) (VSCode)

## Features

- Autocomplete for names and versions
- Hover for information - includes description, links to documentation & more
- Diagnostics:
  - A newer version is available
  - The specified tool / package does not exist
  - Unsupported platform / architecture / name
- Quick actions on diagnostics - update to newest version

## TODO

<details>
<summary>Cargo</summary>

Not yet implemented:

- Autocomplete for dependencies - features

</details>

<details>
<summary>Foreman</summary>

Not yet implemented:

- Everything supported by Aftman

</details>

## Development

The VSCode extension can be compiled and installed locally:

1. Clone the repository
2. Install [Just], [Rust], [VSCE] and the [VSCode CLI]
3. Run `just vscode-install` in the repository to install the extension

[Just]: https://github.com/casey/just
[Rust]: https://www.rust-lang.org/tools/install
[VSCE]: https://github.com/microsoft/vscode-vsce
[VSCode CLI]: https://code.visualstudio.com/docs/editor/command-line
