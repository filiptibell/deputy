# Zed Tooling Language Server

## Usage
Zed Tooling Language Server uses a binary named `tooling-language-server` if it is available in
the current worktree, and otherwise falls back to installing the latest version of the language
server automatically and using that.

The extension supports the following slash commands:
* `/tooling-language-server-set-github-personal-access-token <personal-access-token>`: Sets the
  GitHub personal access token to use for GitHub API requests. This prevents rate limiting and is
  required for the language server to work if you're using a private Wally index repository.
* `/tooling-language-server-remove-github-personal-access-token`: Removes the GitHub personal access
  token that was previously set.
