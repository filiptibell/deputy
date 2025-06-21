# Deputy

A language server for your dependencies.

Deputy currently supports the following package managers & toolchain managers:

- [Cargo](https://crates.io) (`Cargo.toml`)
- [NPM](https://www.npmjs.com) (`package.json`)
- [Rokit](https://github.com/rojo-rbx/rokit) (`rokit.toml`)
- [Wally](https://github.com/UpliftGames/wally) (`wally.toml`)

Provides autocomplete, diagnostics for out-of-date versions, and more. <br/>
Check out the [features](#features) section for a full list of features.

## Features

- Autocomplete for names, versions, and features
- Hover for information - includes description, links to documentation & more
- Diagnostics:
  - A newer version is available
  - The specified tool / package / version does not exist
- Quick actions on diagnostics - update to newest version

## Usage

The Deputy extension uses a binary named `deputy` if it is available in
the current worktree, and otherwise falls back to installing the latest
version of the language server automatically and using that.

The extension also adds the following slash commands:

* `/deputy-set-github-pat <personal-access-token>`: Sets the GitHub personal access token to use for GitHub API requests.
  This prevents rate limiting and is required for the language server to work if you're using a private Wally index repository.
* `/deputy-remove-github-pat`: Removes the GitHub personal access token that was previously set.
