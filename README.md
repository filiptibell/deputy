# Tooling Language Server

A language server for several tools:

- [Aftman](https://github.com/LPGhatguy/aftman)
- [Foreman](https://github.com/roblox/foreman)
- [Wally](https://github.com/UpliftGames/wally)

This is mostly a small personal project to learn how to write a performant language server and lexer/parser for TOML files.

## Aftman

Features that are currently supported:

- Diagnostics for:
  - A newer tool version is available
  - Invalid author / name / version
- Hover for information about a tool (description)
- Quick action to update to new tool version

Features that will be supported:

- Diagnostic for unsupported platform/arch
- Autocomplete for tools under a specific GitHub user/org
- All of the listed Aftman features for Foreman as well

## Wally

Features that will be supported:

- Diagnostics for invalid package names and/or realms
- Autocomplete for dependencies - scopes + names + versions
- Hover for more information about a package
- Inlay hints and quick actions for new package versions
