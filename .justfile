EXT := if os() == "windows" { ".exe" } else { "" }
CWD := invocation_directory()
VSCODE := invocation_directory() / "editors" / "vscode"
BIN_NAME := "tooling-language-server"

# Default hidden recipe for listing other recipes + cwd
[no-cd]
[no-exit-message]
[private]
default:
	#!/usr/bin/env bash
	set -euo pipefail
	printf "Current directory:\n    {{CWD}}\n"
	just --list

# Builds the language server
[no-exit-message]
build *ARGS:
	#!/usr/bin/env bash
	set -euo pipefail
	cargo build --bin {{BIN_NAME}} {{ARGS}}

# Copies the built language server into the VSCode (VSIX) extension build directory
[no-exit-message]
[private]
vsix-copy-files TARGET_DIR DEBUG="false":
	#!/usr/bin/env bash
	set -euo pipefail
	#
	rm -rf "{{VSCODE}}/out"
	rm -rf "{{VSCODE}}/bin"
	rm -rf "{{VSCODE}}/CHANGELOG.md"
	rm -rf "{{VSCODE}}/LICENSE.txt"
	mkdir -p "{{VSCODE}}/out"
	mkdir -p "{{VSCODE}}/bin"
	#
	if [[ "{{DEBUG}}" == "true" ]]; then
		mkdir -p {{VSCODE}}/out/debug/
		cp {{TARGET_DIR}}/debug/{{BIN_NAME}}{{EXT}} {{VSCODE}}/out/debug/
	else
		mkdir -p {{VSCODE}}/out/release/
		cp {{TARGET_DIR}}/release/{{BIN_NAME}}{{EXT}} {{VSCODE}}/out/release/
	fi
	#
	cp CHANGELOG.md {{VSCODE}}/CHANGELOG.md
	cp LICENSE.txt {{VSCODE}}/LICENSE.txt

# Packages the VSCode (VSIX) extension - vsix-copy-files must be used first
[no-exit-message]
[private]
vsix-package:
	#!/usr/bin/env bash
	set -euo pipefail
	cd "{{VSCODE}}/"
	bun install
	vsce package --out "{{VSCODE}}/bin/"

# Fully builds the language server + VSIX (VSCode + Open VSX) extension for publishing
[no-exit-message]
vsix-build TARGET_TRIPLE:
	#!/usr/bin/env bash
	set -euo pipefail
	#
	echo "🚧 Building language server..."
	just build --release --target {{TARGET_TRIPLE}}
	echo "📦 Copying language server files..."
	just vsix-copy-files "target/{{TARGET_TRIPLE}}"
	echo "🧰 Building extension..."
	just vsix-package

# Builds and publishes the VSIX (VSCode + Open VSX) extension to marketplaces
[no-exit-message]
vsix-publish TARGET_TRIPLE EXTENSION_TARGET:
	#!/usr/bin/env bash
	set -euo pipefail
	#
	just vsix-build {{TARGET_TRIPLE}}
	#
	echo "🚀 Publishing extension..."
	cd "{{VSCODE}}/"
	vsce publish --target {{EXTENSION_TARGET}}
	ovsx publish --target {{EXTENSION_TARGET}}
	echo "✅ Published extension successfully!"

# Builds and installs the VSCode extension locally
[no-exit-message]
vscode-install DEBUG="false":
	#!/usr/bin/env bash
	set -euo pipefail
	#
	echo "🚧 Building language server..."
	if [[ "{{DEBUG}}" == "true" ]]; then
		just build
	else
		just build --release
	fi
	echo "📦 Copying language server files..."
	just vsix-copy-files "target" {{DEBUG}} > /dev/null
	echo "🧰 Building extension..."
	just vsix-package > /dev/null
	echo "🚀 Installing extension..."
	#
	EXTENSION=$(find "{{VSCODE}}/bin/" -name "*.vsix")
	code --install-extension "$EXTENSION" &> /dev/null
	#
	echo "✅ Installed extension successfully!"

# Zips up language server and built extensions into single zip file
[no-exit-message]
zip-release TARGET_TRIPLE:
	#!/usr/bin/env bash
	set -euo pipefail
	rm -rf staging
	rm -rf release.zip
	mkdir -p staging
	cp "target/{{TARGET_TRIPLE}}/release/{{BIN_NAME}}{{EXT}}" staging/
	cp "$(find "{{VSCODE}}/bin/" -name "*.vsix")" staging/extension.vsix
	cd staging
	if [ "{{os_family()}}" = "windows" ]; then
		7z a ../release.zip *
	else
		chmod +x {{BIN_NAME}}
		zip ../release.zip *
	fi
	cd "{{CWD}}"
	rm -rf staging

# Used in GitHub workflow to move per-matrix release zips
[no-exit-message]
[private]
unpack-releases RELEASES_DIR:
	#!/usr/bin/env bash
	set -euo pipefail
	#
	if [ ! -d "{{RELEASES_DIR}}" ]; then
		echo "Releases directory is missing"
		exit 1
	fi
	#
	cd "{{RELEASES_DIR}}"
	echo ""
	echo "Releases dir:"
	ls -lhrt
	echo ""
	echo "Searching for zipped releases..."
	#
	for DIR in * ; do
		if [ -d "$DIR" ]; then
			cd "$DIR"
			for FILE in * ; do
				if [ ! -d "$FILE" ]; then
					if [ "$FILE" = "release.zip" ]; then
						echo "Found zipped release '$DIR'"
						mv "$FILE" "../$DIR.zip"
						rm -rf "../$DIR/"
					fi
				fi
			done
			cd ..
		fi
	done
	#
	echo ""
	echo "Releases dir:"
	ls -lhrt
