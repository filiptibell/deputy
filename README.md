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
- All of the listed Aftman features for Foreman as well

</details>

<details>
<summary>Cargo</summary>

Features that are currently supported:

- Hover for information about a package (description, links)
- Diagnostics for:
  - A newer package version is available
  - Invalid package name / version
- Quick action to update to a new package version

Features that will be supported:

- Autocomplete for versions, features

</details>

<details>
<summary>Wally</summary>

Features that will be supported:

- Diagnostics for:
  - A newer package version is available
  - Invalid author / name / version
  - Invalid package realm
- Autocomplete for packages - authors + names + versions
- Hover for information about a package (description, links)
- Quick action to update to a new package version

</details>

## Development

The VSCode extension can be compiled and installed locally:

1. Clone the repository
2. Install [Just], [VSCE] and the [VSCode CLI]
3. Run `just vscode-install` in the repository to install the extension

[Just]: https://github.com/casey/just
[VSCE]: https://github.com/microsoft/vscode-vsce
[VSCode CLI]: https://code.visualstudio.com/docs/editor/command-line
