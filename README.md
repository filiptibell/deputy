# Tooling Language Server

A language server for several tools:

- [Aftman](https://github.com/LPGhatguy/aftman)
- [Foreman](https://github.com/roblox/foreman)
- [Wally](https://github.com/UpliftGames/wally)

This is mostly a small personal project to learn how to write a performant language server and lexer/parser for TOML files.

## Aftman / Foreman

Features that will be supported:

- Diagnostics for invalid tool names, unsupported platform
- Autocomplete for tools under a specific GitHub user or organization
- Hover for more information about a tool
- Inlay hints and quick actions for new tool versions

## Wally

Features that will be supported:

- Diagnostics for invalid package names and/or realms
- Autocomplete for dependencies - scopes + names + versions
- Hover for more information about a package
- Inlay hints and quick actions for new package versions
