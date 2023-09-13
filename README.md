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

A language server for several tools - [Aftman], [Cargo], [Foreman] and [Wally].

Provides autocomplete for tools & dependencies, diagnostics for out-of-date versions, and more.

[Aftman]: https://github.com/LPGhatguy/aftman
[Cargo]: https://crates.io
[Foreman]: https://github.com/roblox/foreman
[Wally]: https://github.com/UpliftGames/wally

## Features

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
