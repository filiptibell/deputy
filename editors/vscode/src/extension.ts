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

import { promptAuthForGitHub } from "./auth";
import { fileExists } from "./fs";
import { RateLimitNotification } from "./notifications";

let client: LanguageClient;
let context: vscode.ExtensionContext;
let outputChannel: vscode.OutputChannel;

const GITHUB_AUTH_TOKEN_STORAGE_KEY = "auth.github.token";

const sendAuthForGitHub = async () => {
	const auth = await promptAuthForGitHub();
	if (!auth) {
		outputChannel.appendLine("AUTH GitHub token prompt was dismissed");
		return;
	}

	const notification: RateLimitNotification = {
		kind: "GitHub",
		value: auth,
	};

	outputChannel.appendLine("AUTH saving GitHub token to global context");
	await context.globalState.update(GITHUB_AUTH_TOKEN_STORAGE_KEY, auth);

	outputChannel.appendLine("AUTH sending GitHub token to server");
	client.sendNotification("$/internal_message/rate_limit", notification);
};

export async function activate(extensionContext: vscode.ExtensionContext) {
	context = extensionContext;

	// Try to load previously stored auth
	let githubAuthToken = context.globalState.get(
		GITHUB_AUTH_TOKEN_STORAGE_KEY
	);

	// Create a persistent output channel to use
	outputChannel = vscode.window.createOutputChannel(
		"Tooling Language Server"
	);

	// Find which executable was bundled with the extension - either debug or release

	let exeName =
		os.platform() === "win32"
			? "tooling-language-server.exe"
			: "tooling-language-server";

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

	let isDebug = command === exeDebug.fsPath;
	let options: ExecutableOptions = isDebug
		? { env: { RUST_LOG: "debug", RUST_BACKTRACE: "1" } }
		: { env: {} };
	if (typeof githubAuthToken === "string" && githubAuthToken.length > 0) {
		outputChannel.appendLine(" AUTH found stored GitHub token");
		options.env["GITHUB_TOKEN"] = githubAuthToken;
	}
	let server: Executable = {
		command,
		options,
	};

	// Create language server & client config

	let serverOptions: ServerOptions = {
		run: server,
		debug: server,
	};

	let clientOptions: LanguageClientOptions = {
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

	// Listen for custom notifications from server
	client.onNotification(
		"$/internal_message/rate_limit",
		(value: RateLimitNotification) => {
			if (value.kind === "GitHub") {
				sendAuthForGitHub();
			}
		}
	);
}

export function deactivate() {
	if (client) {
		client.stop();
	}
}
