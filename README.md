<!-- markdownlint-disable MD033 -->
<!-- markdownlint-disable MD041 -->

<h1 align="center">Deputy</h1>

<div align="center">
  <a href="https://github.com/filiptibell/deputy/actions">
  <img src="https://shields.io/endpoint?url=https://badges.readysetplay.io/workflow/filiptibell/deputy/ci.yaml" alt="CI status" />
  </a>
  <a href="https://github.com/filiptibell/deputy/actions">
    <img src="https://shields.io/endpoint?url=https://badges.readysetplay.io/workflow/filiptibell/deputy/release.yaml" alt="Release status" />
  </a>
  <a href="https://github.com/filiptibell/deputy/blob/main/LICENSE.txt">
    <img src="https://img.shields.io/github/license/filiptibell/deputy.svg?label=License&color=informational" alt="Language server license" />
  </a>
</div>

<br/>

A language server for your dependencies.

Deputy currently supports the following package managers & toolchain managers:

- [Cargo](https://crates.io) (`Cargo.toml`)
- [NPM](https://www.npmjs.com) (`package.json`)
- [Rokit](https://github.com/rojo-rbx/rokit) (`rokit.toml`)
- [Wally](https://github.com/UpliftGames/wally) (`wally.toml`)

Provides autocomplete, diagnostics for out-of-date versions, and more. <br/>
Check out the [features](#features) and [screenshots](#screenshots) sections for a full overview.

## Installation

Deputy can be installed as an extension from:

- The [Visual Studio Marketplace](https://marketplace.visualstudio.com/items?itemName=filiptibell.deputy) (VSCode)
- The [Open VSX Registry](https://open-vsx.org/extension/filiptibell/deputy) (Cursor, Windsurf, ...)

The language server can also be installed manually from the [latest release](https://github.com/filiptibell/deputy/releases/latest).

<details>
<summary> Manual Installation - VSCode </summary>

1. [Install Bun](https://bun.sh/docs/installation)
2. [Install the VSCode Command Line Interface](https://code.visualstudio.com/docs/editor/command-line)
3. Make sure you have installed the language server binary and that it exists on PATH
4. Clone this repository, and navigate to the `editors/vscode` directory
5. Finally, build and install the extension by running these three commands, in order:
   ```bash
   bun install
   bun pm trust --all
   bun run extension-install
   ```

</details>

<details>
<summary> Manual Installation - Zed </summary>

1. [Install Rust](https://www.rust-lang.org/tools/install)
2. Make sure you have installed the language server binary and that it exists on PATH
3. Clone this repository, and navigate to the root directory
4. Install the Zed extension at `editors/zed` as a [dev extension](https://zed.dev/docs/extensions/developing-extensions#developing-an-extension-locally)

</details>

## Features

- Autocomplete for names, versions, and features
- Hover for information - includes description, links to documentation & more
- Diagnostics:
  - A newer version is available
  - The specified tool / package / version does not exist
- Quick actions on diagnostics - update to newest version

## Screenshots

### Hovers

<img src="assets/cargo-screenshot-hover.png" alt="Hover" height="50%" width="50%" />

### Diagnostics

<img src="assets/cargo-screenshot-diagnostics.png" alt="Diagnostics" height="50%" width="50%" />

### Completions

<img src="assets/cargo-screenshot-completions.png" alt="Completions" height="50%" width="50%" />


## Why "Deputy"?

This project was previously called "Tooling Language Server", but I felt it was a bit too generic and verbose.

Deputy is mostly just a fun wordplay on "Dependency" and "Utility", but you can also interpret it using the literal meaning of the word. <br/>
Deputy helps you (the sheriff of dependencies) stay informed, and keep your versions up to date. 🤠
