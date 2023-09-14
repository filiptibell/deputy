EXT := if os() == "windows" { ".exe" } else { "" }
CWD := justfile_directory()
VSCODE := justfile_directory() / "editors/vscode"
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

# Packs the language server into the VSCode extension build directory
[no-exit-message]
[private]
vscode-pack TARGET_DIR DEBUG="false":
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

# Builds the VSCode extension - must be used after vscode-pack
[no-exit-message]
[private]
vscode-build:
	#!/usr/bin/env bash
	set -euo pipefail
	cd "{{VSCODE}}/"
	npm install
	vsce package --out "{{VSCODE}}/bin/"

# Builds and installs the VSCode extension locally
[no-exit-message]
vscode-install DEBUG="false":
	#!/usr/bin/env bash
	set -euo pipefail
	#
	echo "🚧 [1/4] Building language server..."
	if [[ "{{DEBUG}}" == "true" ]]; then
		just build
	else
		just build --release
	fi
	echo "📦 [2/4] Packing language server..."
	just vscode-pack "target" {{DEBUG}} > /dev/null
	echo "🧰 [3/4] Building extension..."
	just vscode-build > /dev/null
	echo "🚀 [4/4] Installing extension..."
	#
	EXTENSION=$(find "{{VSCODE}}/bin/" -name "*.vsix")
	code --install-extension "$EXTENSION" &> /dev/null
	#
	echo "✅ Installed extension successfully!"

# Builds and publishes the VSCode extension to the marketplace
[no-exit-message]
vscode-publish TARGET_TRIPLE VSCODE_TARGET:
	#!/usr/bin/env bash
	set -euo pipefail
	#
	echo "🚧 [1/4] Building language server..."
	just build --release --target {{TARGET_TRIPLE}}
	echo "📦 [2/4] Packing language server..."
	just vscode-pack "target/{{TARGET_TRIPLE}}"
	echo "🧰 [3/4] Building extension..."
	just vscode-build
	echo "🚀 [4/4] Publishing extension..."
	#
	cd "{{VSCODE}}/"
	vsce publish --target {{VSCODE_TARGET}}
	#
	echo "✅ Published extension successfully!"

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
	if [ "{{os_family()}}" = "windows-latest" ]; then
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
unpack-releases:
	#!/usr/bin/env bash
	set -euo pipefail
	#
	if [ ! -d "releases" ]; then
		echo "Releases directory is missing"
		exit 1
	fi
	#
	cd releases
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
	cd ..
