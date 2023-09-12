ext := if os() == "windows" { ".exe" } else { "" }

# Builds the language server
[no-exit-message]
build DEBUG="false":
	#!/usr/bin/env bash
	set -euo pipefail
	if [[ "{{DEBUG}}" == "true" ]]; then
		cargo build --bin tooling-language-server
	else
		cargo build --bin tooling-language-server --release > /dev/null
	fi

# Bundles the language server into the VSCode extension build directory
[no-exit-message]
vscode-bundle DEBUG="false":
	#!/usr/bin/env bash
	set -euo pipefail
	if [[ "{{DEBUG}}" == "true" ]]; then
		mkdir -p ./editors/vscode/out/debug/
		cp target/debug/tooling-language-server{{ext}} ./editors/vscode/out/debug/
	else
		mkdir -p ./editors/vscode/out/release/
		cp target/release/tooling-language-server{{ext}} ./editors/vscode/out/release/
	fi

# Cleans up artifacts from building the VSCode extension
[no-exit-message]
vscode-cleanup:
	#!/usr/bin/env bash
	set -euo pipefail
	cd "./editors/vscode/"
	WORKDIR="$PWD"
	rm -rf "$WORKDIR/out"
	rm -rf "$WORKDIR/bin"
	mkdir -p "$WORKDIR/bin"
	cd "../../"

# Builds the VSCode extension
[no-exit-message]
vscode-build:
	#!/usr/bin/env bash
	set -euo pipefail
	cd "./editors/vscode/"
	WORKDIR="$PWD"
	vsce package --out "$WORKDIR/bin/" > /dev/null
	cd "../../"

# Builds and installs the VSCode extension locally
[no-exit-message]
vscode-install DEBUG="false":
	#!/usr/bin/env bash
	set -euo pipefail
	#
	echo "🚧 [1/4] Building language server..."
	just build {{DEBUG}}
	echo "📦 [2/4] Packing language server..."
	just vscode-cleanup
	just vscode-bundle {{DEBUG}}
	echo "🧰 [3/4] Building extension..."
	just vscode-build
	echo "🚀 [4/4] Installing extension..."
	#
	cd "./editors/vscode/"
	WORKDIR="$PWD"
	EXTENSION=$(find "$WORKDIR/bin/" -name "*.vsix")
	code --install-extension "$EXTENSION" > /dev/null
	cd "../../"
	#
	echo "✅ Installed extension successfully!"

# Builds and publishes the VSCode extension to the marketplace
[no-exit-message]
vscode-publish: vscode-build
	#!/usr/bin/env bash
	set -euo pipefail
	#
	echo "🚧 [1/4] Building language server..."
	just build
	echo "📦 [2/4] Packing language server..."
	just vscode-cleanup
	just vscode-bundle
	echo "🧰 [3/4] Building extension..."
	just vscode-build
	echo "🚀 [4/4] Publishing extension..."
	#
	cd "./editors/vscode/"
	vsce publish > /dev/null
	cd "../../"
	#
	echo "✅ Published extension successfully!"
