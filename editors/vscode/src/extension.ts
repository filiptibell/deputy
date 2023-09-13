/* eslint-disable @typescript-eslint/naming-convention */

import * as vscode from "vscode";
import * as os from "os";

import {
	Executable,
	ExecutableOptions,
	LanguageClient,
	LanguageClientOptions,
	ServerOptions,
} from "vscode-languageclient/node";

import { getAuthForGitHub } from "./auth";
import { fileExists } from "./fs";
import {
	RATE_LIMIT_METHOD,
	handleRateLimitRequest,
	RateLimitRequest,
} from "./requests";

let client: LanguageClient;

export async function activate(context: vscode.ExtensionContext) {
	const outputChannel = vscode.window.createOutputChannel(
		"Tooling Language Server"
	);

	// Find which executable was bundled with the extension - either debug or release

	const exeName =
		os.platform() === "win32"
			? "tooling-language-server.exe"
			: "tooling-language-server";

	const exeDebug = vscode.Uri.joinPath(
		context.extensionUri,
		"out",
		"debug",
		exeName
	);

	const exeRelease = vscode.Uri.joinPath(
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

	// Create args for language server, including any stored auth

	const isDebug = command === exeDebug.fsPath;
	const options: ExecutableOptions = isDebug
		? { env: { RUST_LOG: "debug", RUST_BACKTRACE: "1" } }
		: { env: {} };

	options.env["GITHUB_TOKEN"] = getAuthForGitHub(context);

	const server: Executable = {
		command,
		options,
	};

	// Create language server & client config

	const serverOptions: ServerOptions = {
		run: server,
		debug: server,
	};

	const clientOptions: LanguageClientOptions = {
		stdioEncoding: "utf8",
		documentSelector: [{ scheme: "file", language: "toml" }],
		diagnosticCollectionName: "Tooling Language Server",
		outputChannel,
	};

	// Start the language client

	if (isDebug) {
		outputChannel.appendLine("Starting language server (debug)");
	} else {
		outputChannel.appendLine("Starting language server");
	}

	client = new LanguageClient(
		"tooling-language-server",
		"Tooling Language Server",
		serverOptions,
		clientOptions
	);

	client.start();

	// Listen for custom requests from server
	client.onRequest(RATE_LIMIT_METHOD, async (request: RateLimitRequest) => {
		return await handleRateLimitRequest(context, request);
	});
}

export function deactivate() {
	if (client) {
		client.stop();
	}
}
