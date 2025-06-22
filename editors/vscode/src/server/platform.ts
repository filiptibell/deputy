import * as os from "os";
import { BINARY_NAME, BINARY_ROOT_DIR } from "./constants";

// Types

export type OS = "windows" | "macos" | "linux";
export type Arch = "x86_64" | "aarch64";

// Constructor partials

function constructOs(): OS {
	switch (os.platform()) {
		case "win32":
			return "windows";
		case "darwin":
			return "macos";
		case "linux":
			return "linux";
		default:
			throw new Error(`Unsupported OS: ${os.platform()}`);
	}
}

function constructArch(): Arch {
	switch (os.arch()) {
		case "x64":
			return "x86_64";
		case "arm64":
			return "aarch64";
		default:
			throw new Error(`Unsupported architecture: ${os.arch()}`);
	}
}

function constructExeSuffix(): string {
	return os.platform() === "win32" ? ".exe" : "";
}

// Strings class

export class PlatformDescriptor {
	private readonly os: OS;
	private readonly arch: Arch;
	private readonly exeSuffix: string;

	constructor() {
		this.os = constructOs();
		this.arch = constructArch();
		this.exeSuffix = constructExeSuffix();
	}

	public releaseAssetName(version: string): string {
		const cleanVersion = version.startsWith("v")
			? version.slice(1)
			: version;
		return `${BINARY_NAME}-${cleanVersion}-${this.os}-${this.arch}.zip`;
	}

	public serverBinaryRoot(): string {
		return BINARY_ROOT_DIR;
	}

	public serverBinaryDir(version: string): string {
		return `${BINARY_NAME}-${version}`;
	}

	public serverBinaryPath(): string {
		return `${BINARY_NAME}${this.exeSuffix}`;
	}

	public isUnix(): boolean {
		return this.os === "linux" || this.os === "macos";
	}
}
