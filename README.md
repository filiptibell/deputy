# Tooling Language Server

A language server for several tools:

- [Aftman](https://github.com/LPGhatguy/aftman)
- [Foreman](https://github.com/roblox/foreman)
- [Wally](https://github.com/UpliftGames/wally)

This is mostly a small personal project to learn how to write a performant language server and lexer/parser for TOML files.

## Aftman / Foreman

Features that are currently supported:

- Diagnostics for invalid tool names, new tool versions
- Hover for more information about a tool (description)

Features that will be supported:

- Diagnostic for unsupported platform/arch
- Autocomplete for tools under a specific GitHub user/org
- Inlay hints and quick action to update to new tool version

## Wally

Features that will be supported:

- Diagnostics for invalid package names and/or realms
- Autocomplete for dependencies - scopes + names + versions
- Hover for more information about a package
- Inlay hints and quick actions for new package versions
