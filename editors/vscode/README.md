# Deputy

A lightweight language server for your dependencies.

Deputy currently supports the following:

- [Cargo](https://crates.io) (`Cargo.toml`)
- [Golang](https://pkg.go.dev) (`go.mod`)
- [NPM](https://www.npmjs.com) (`package.json`)
- [Python](https://pypi.org) (`pyproject.toml`)
- [Rokit](https://github.com/rojo-rbx/rokit) (`rokit.toml`)
- [Wally](https://github.com/UpliftGames/wally) (`wally.toml`)

Provides autocomplete, diagnostics for out-of-date versions, and more.

Check out the [features](#features) section for a full list of features.

## Features

- Autocomplete for names, versions, and features
- Hover for information - includes description, links to documentation & more
- Diagnostics:
  - A newer version is available
  - The specified tool / package / version does not exist
- Quick actions on diagnostics - update to newest version
