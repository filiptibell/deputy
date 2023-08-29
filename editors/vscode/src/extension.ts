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

export async function activate(context: vscode.ExtensionContext) {
	let exeName = os.platform() === "win32" ? "server.exe" : "server";

	let run: Executable = {
		transport: TransportKind.stdio,
		command: vscode.Uri.joinPath(
			context.extensionUri,
			"out",
			"release",
			exeName
		).fsPath,
	};

	let debug: Executable = {
		transport: TransportKind.stdio,
		command: vscode.Uri.joinPath(
			context.extensionUri,
			"out",
			"debug",
			exeName
		).fsPath,
		args: ["RUST_LOG=debug"],
	};

	let serverOptions: ServerOptions = { run, debug };
	let clientOptions: LanguageClientOptions = {
		documentSelector: [{ scheme: "file", language: "toml" }],
		synchronize: {
			fileEvents: vscode.workspace.createFileSystemWatcher("**/*.toml"),
		},
	};

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
