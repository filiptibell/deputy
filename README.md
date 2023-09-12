<!-- markdownlint-disable MD033 -->

# Tooling Language Server

A language server for several tools - [Aftman], [Cargo], [Foreman] and [Wally]. <br/>
Personal / free time project, to learn how to write a language server.

[Aftman]: https://github.com/LPGhatguy/aftman
[Cargo]: https://crates.io
[Foreman]: https://github.com/roblox/foreman
[Wally]: https://github.com/UpliftGames/wally

## Tools

<details>
<summary>Aftman</summary>

Features that are currently supported:

- Diagnostics for:
  - A newer tool version is available
  - Invalid author / name / version
- Hover for information about a tool (description, links)
- Autocomplete for commonly used tool authors & names, versions
- Quick action to update to new tool version

Features that will be supported:

- Diagnostic for unsupported platform/arch

</details>

<details>
<summary>Cargo</summary>

Features that are currently supported:

- Hover for information about a dependency (description, links)
- Diagnostics for:
  - A newer dependency version is available
  - Invalid dependency name / version
- Quick action to update to a new dependency version

Features that will be supported:

- Autocomplete for dependencies - versions, features

</details>

<details>
<summary>Foreman</summary>

See the Aftman section.

All features supported by Aftman will also be supported for Foreman.

</details>

<details>
<summary>Wally</summary>

Features that are currently supported:

- Diagnostics for:
  - A newer dependency version is available
  - Invalid author / name / version
- Hover for information about a dependency (description, links)
- Autocomplete for dependencies - authors + names + versions
- Quick action to update to a new dependency version

Features that will be supported:

- Diagnostics for:
  - Invalid dependency realm

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
