<!-- markdownlint-disable MD033 -->
<!-- markdownlint-disable MD041 -->

<h1 align="center">Tooling Language Server</h1>

<div align="center">
  <a href="https://marketplace.visualstudio.com/items?itemName=filiptibell.tooling-language-server">
    <img src="https://vsmarketplacebadges.dev/version/filiptibell.tooling-language-server.png" alt="VSCode Marketplace" />
  </a>
</div>

<br/>

A language server supporting several tools and package managers:

- [Aftman](https://github.com/LPGhatguy/aftman) / [Foreman](https://github.com/roblox/foreman)
- [Cargo](https://crates.io)
- [NPM](https://www.npmjs.com)
- [Wally](https://github.com/UpliftGames/wally)

Provides autocomplete, diagnostics for out-of-date versions, and more. <br/>
Check out the [features](#features) section for a full list of features.

## Features

- Autocomplete for names and versions
- Hover for information - includes description, links to documentation & more
- Diagnostics:
  - A newer version is available
  - The specified tool / package does not exist
  - Unsupported platform / architecture / name
- Quick actions on diagnostics - update to newest version
