[no-exit-message]
build-vscode-extension:
	#!/usr/bin/env bash
	set -euo pipefail
	cd "./editors/vscode/"
	WORKDIR="$PWD"
	rm -rf "$WORKDIR/out"
	rm -rf "$WORKDIR/bin"
	mkdir -p "$WORKDIR/bin"
	echo "🛠️  Building extension..."
	vsce package --out "$WORKDIR/bin/" > /dev/null
	cd "../../"

[no-exit-message]
install-vscode-extension: build-vscode-extension
	#!/usr/bin/env bash
	set -euo pipefail
	cd "./editors/vscode/"
	WORKDIR="$PWD"
	EXTENSION=$(find "$WORKDIR/bin/" -name "*.vsix")
	echo "🚀 Installing extension..."
	code --install-extension "$EXTENSION" > /dev/null
	echo "✅ Installed extension successfully!"
	cd "../../"

[no-exit-message]
publish-vscode-extension:
	#!/usr/bin/env bash
	set -euo pipefail
	cd "./editors/vscode/"
	echo "🛠️  Publishing extension..."
	vsce publish > /dev/null
	echo "✅ Published extension successfully!"
	cd "../../"
