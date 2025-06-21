<!-- markdownlint-disable MD033 -->
<!-- markdownlint-disable MD041 -->

<h1 align="center">Deputy</h1>

<div align="center">
  <a href="https://marketplace.visualstudio.com/items?itemName=filiptibell.deputy">
  <img src="https://vsmarketplacebadges.dev/version/filiptibell.deputy.png" alt="VSCode Marketplace" />
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
Check out the [features](#features) section for a full list of features.

## Features

- Autocomplete for names, versions, and features
- Hover for information - includes description, links to documentation & more
- Diagnostics:
  - A newer version is available
  - The specified tool / package / version does not exist
- Quick actions on diagnostics - update to newest version
