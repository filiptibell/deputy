ext := if os() == "windows" { ".exe" } else { "" }

[no-exit-message]
build-server DEBUG="false":
	#!/usr/bin/env bash
	set -euo pipefail
	if [[ "{{DEBUG}}" == "true" ]]; then
		echo "🖥️  Building language server... (debug)"
		cargo build --bin server
		mkdir -p ./editors/vscode/out/debug/
		cp target/debug/server{{ext}} ./editors/vscode/out/debug/
	else
		echo "🖥️  Building language server..."
		cargo build --bin server --release
		mkdir -p ./editors/vscode/out/release/
		cp target/release/server{{ext}} ./editors/vscode/out/release/
	fi
	echo "✅ Built language server successfully!"

[no-exit-message]
cleanup-vscode-artifacts:
	#!/usr/bin/env bash
	set -euo pipefail
	cd "./editors/vscode/"
	WORKDIR="$PWD"
	rm -rf "$WORKDIR/out"
	rm -rf "$WORKDIR/bin"
	mkdir -p "$WORKDIR/bin"
	cd "../../"

[no-exit-message]
build-vscode-extension DEBUG="false": cleanup-vscode-artifacts (build-server DEBUG)
	#!/usr/bin/env bash
	set -euo pipefail
	cd "./editors/vscode/"
	WORKDIR="$PWD"
	echo "🛠️  Building extension..."
	vsce package --out "$WORKDIR/bin/" > /dev/null
	cd "../../"

[no-exit-message]
install-vscode-extension DEBUG="false": (build-vscode-extension DEBUG)
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
publish-vscode-extension: build-server
	#!/usr/bin/env bash
	set -euo pipefail
	cd "./editors/vscode/"
	echo "🛠️  Publishing extension..."
	vsce publish > /dev/null
	echo "✅ Published extension successfully!"
	cd "../../"
