import * as vscode from "vscode";
import * as os from "os";

import {
	Executable,
	LanguageClient,
	LanguageClientOptions,
	ServerOptions,
	TransportKind,
} from "vscode-languageclient/node";

let client: LanguageClient;

const fileExists = async (path: vscode.Uri): Promise<boolean> => {
	try {
		let stat = await vscode.workspace.fs.stat(path);
		return stat.type === vscode.FileType.File;
	} catch {
		return false;
	}
};

export async function activate(context: vscode.ExtensionContext) {
	// Create a persistent output channel to use

	let outputChannel = vscode.window.createOutputChannel(
		"Tooling Language Server"
	);

	outputChannel.appendLine("Starting language server");

	// Find which executable was bundled with the extension - either debug or release

	let exeName = os.platform() === "win32" ? "server.exe" : "server";

	let exeDebug = vscode.Uri.joinPath(
		context.extensionUri,
		"out",
		"debug",
		exeName
	);

	let exeRelease = vscode.Uri.joinPath(
		context.extensionUri,
		"out",
		"release",
		exeName
	);

	const command = (await fileExists(exeRelease))
		? exeRelease.fsPath
		: (await fileExists(exeDebug))
		? exeDebug.fsPath
		: null;
	if (!command) {
		vscode.window.showErrorMessage("Missing language server executable");
		return;
	}

	// Create args for language server

	let args = command === exeDebug.fsPath ? ["RUST_LOG=debug"] : [];
	let server: Executable = {
		transport: TransportKind.stdio,
		command,
		args,
	};

	// Create language server & client config

	let serverOptions: ServerOptions = {
		run: server,
		debug: server,
	};

	let clientOptions: LanguageClientOptions = {
		stdioEncoding: "utf8",
		documentSelector: [{ scheme: "file", language: "toml" }],
		outputChannel,
		diagnosticPullOptions: {
			onChange: true,
			onSave: true,
		},
		synchronize: {
			fileEvents: [
				vscode.workspace.createFileSystemWatcher("aftman.toml"),
				vscode.workspace.createFileSystemWatcher("foreman.toml"),
				vscode.workspace.createFileSystemWatcher("wally.toml"),
			],
		},
	};

	// Start the language client

	client = new LanguageClient(
		"tooling-language-server",
		"Tooling Language Server",
		serverOptions,
		clientOptions
	);

	client.start();
}

export function deactivate() {
	if (client) {
		client.stop();
	}
}
