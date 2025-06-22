import * as fs from "fs/promises";
import * as vscode from "vscode";
import JSZip from "jszip";

import { GITHUB_REPO } from "./constants";
import { PlatformDescriptor } from "./platform";

// Types

type GithubReleaseAsset = {
	name: string;
	browser_download_url: string;
};

type GithubRelease = {
	tag_name: string;
	assets: GithubReleaseAsset[];
};

type ParsedRelease = {
	version: string;
	downloadUrl: string;
};

// Downloader class

export class Downloader {
	private latestVersion: string | null = null;
	private latestDownloaded: boolean = false;

	private readonly pdesc: PlatformDescriptor;

	constructor(
		private readonly context: vscode.ExtensionContext,
		private readonly outputChannel: vscode.OutputChannel,
	) {
		this.pdesc = new PlatformDescriptor();
	}

	// Private

	private dirForVersions(): vscode.Uri {
		return vscode.Uri.joinPath(
			this.context.extensionUri,
			this.pdesc.serverBinaryRoot(),
		);
	}

	private dirForVersion(version: string): vscode.Uri {
		return vscode.Uri.joinPath(
			this.dirForVersions(),
			this.pdesc.serverBinaryDir(version),
		);
	}

	private fileForVersion(version: string): vscode.Uri {
		return vscode.Uri.joinPath(
			this.dirForVersion(version),
			this.pdesc.serverBinaryPath(),
		);
	}

	private async cleanupAllVersionsExcept(
		versionUri: vscode.Uri,
	): Promise<void> {
		const versionsDir = this.dirForVersions();
		try {
			const entries =
				await vscode.workspace.fs.readDirectory(versionsDir);
			for (const [name, type] of entries) {
				const uri = vscode.Uri.joinPath(versionsDir, name);
				if (uri.toString() === versionUri.toString()) {
					continue;
				}
				if (type === vscode.FileType.Directory) {
					this.outputChannel.appendLine(
						`Cleaning up old version (${name})`,
					);
					await vscode.workspace.fs.delete(uri, {
						recursive: true,
						useTrash: false,
					});
				}
			}
		} catch {
			// Ignore if bin directory doesn't exist yet
		}
	}

	private async findLatestDownloadableGithubRelease(): Promise<ParsedRelease> {
		const response = await fetch(
			`https://api.github.com/repos/${GITHUB_REPO}/releases/latest`,
		);

		const data = (await response.json()) as GithubRelease;
		const version = data.tag_name;
		const name = this.pdesc.releaseAssetName(version);

		let downloadUrl: string | null = null;
		for (const asset of data.assets) {
			if (asset.name == name) {
				downloadUrl = asset.browser_download_url;
				break;
			}
		}
		if (!downloadUrl) {
			const assetNames = data.assets.map((a) => a.name).join(", ");
			throw new Error(
				`No release asset was found matching "${name}".` +
					`Found release assets: ${assetNames}`,
			);
		}

		return {
			version,
			downloadUrl,
		};
	}

	private async downloadAndExtractGithubReleaseAsset(
		release: ParsedRelease,
	): Promise<ArrayBuffer> {
		const response = await fetch(release.downloadUrl);

		const zipBytes = await response.arrayBuffer();
		const zipFile = await JSZip.loadAsync(zipBytes);

		const fileName = this.pdesc.serverBinaryPath();
		for (const relativePath in zipFile.files) {
			const entry = zipFile.files[relativePath];
			if (entry.name === fileName) {
				const fileContent = await entry.async("nodebuffer");
				return fileContent;
			}
		}

		throw new Error(
			`Failed to find "${fileName}" in the latest release asset`,
		);
	}

	// Public

	public async download(): Promise<void> {
		if (this.latestDownloaded) {
			return;
		}

		// 1a. Find the latest downloadable GitHub release
		this.outputChannel.appendLine(
			"Finding latest downloadable GitHub release",
		);
		const latest = await this.findLatestDownloadableGithubRelease();
		const dir = this.dirForVersion(latest.version);
		const file = this.fileForVersion(latest.version);

		// 1b. Skip download if the latest version is already downloaded,
		//     making sure to verify that the binary actually exists in
		//     case the binaries directory has been modified somehow
		const existing =
			this.context.globalState.get<string>("downloadedVersion");
		if (existing === latest.version) {
			let dirStats;
			let fileStats;
			try {
				dirStats = await vscode.workspace.fs.stat(dir);
				fileStats = await vscode.workspace.fs.stat(file);
				if (
					dirStats.type === vscode.FileType.Directory &&
					fileStats.type === vscode.FileType.File
				) {
					this.outputChannel.appendLine(
						`Downloaded binary is up to date (${latest.version})`,
					);
					this.latestVersion = latest.version;
					this.latestDownloaded = true;
					return;
				} else {
					throw new Error("Oh no, we need to redownload");
				}
			} catch (_) {}
		}

		// 2. Download the latest version & extract the raw binary from the zip
		this.outputChannel.appendLine(
			`Downloading binary for version ${latest.version}`,
		);
		const binary = await this.downloadAndExtractGithubReleaseAsset(latest);

		// 3a. Write the binary to disk at the correct location
		this.outputChannel.appendLine(`Writing binary to ${file.fsPath}`);
		await vscode.workspace.fs.createDirectory(dir);
		await vscode.workspace.fs.writeFile(file, new Uint8Array(binary));

		// 3b. Make the binary executable on Unix systems, note
		//     that the VSCODE fs API doesn't support chmod
		if (this.pdesc.isUnix()) {
			this.outputChannel.appendLine("Making binary executable");
			await fs.chmod(file.fsPath, 0o755);
		}

		// 4. Update the cache with the downloaded version
		this.latestVersion = latest.version;
		this.latestDownloaded = true;
		this.context.globalState.update("downloadedVersion", latest.version);

		// 5. Finally, remove old versions that are no longer necessary (if any)
		await this.cleanupAllVersionsExcept(dir);
	}

	public path(): string {
		if (this.latestVersion && this.latestDownloaded) {
			return this.fileForVersion(this.latestVersion).fsPath;
		} else {
			throw new Error("Binary has not been downloaded");
		}
	}
}
