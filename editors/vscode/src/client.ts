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

let client: LanguageClient | undefined;
let context: vscode.ExtensionContext;
let outputChannel: vscode.OutputChannel;

/**
	Sets the extension context for the language server.

	This will be used to retrieve stored authentication tokens.
*/
export const setExtensionContext = (ctx: vscode.ExtensionContext) => {
	if (context === undefined) {
		context = ctx;
	} else {
		throw new Error("Extension context can only be set once");
	}
};

/**
	Starts the language server.

	Will throw an error if the language server has already been started.
*/
export const startServer = async () => {
	if (context === undefined) {
		throw new Error("Extension context must be set");
	}
	if (client !== undefined) {
		throw new Error("Language server has already been started");
	}

	// Create persistent output channel if one does not exist

	if (outputChannel === undefined) {
		outputChannel = vscode.window.createOutputChannel(
			"Tooling Language Server"
		);
	}

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

	// Start language server & client

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
};

/**
	Stops the language server.

	Returns `true` if stopped, `false` if the language server was not running.
*/
export const stopServer = async (): Promise<boolean> => {
	const c = client;
	if (c !== undefined) {
		client = undefined;
		await c.stop();
		return true;
	} else {
		return false;
	}
};

/**
	Stops and then starts the language server.

	Should be used only when a language server configuration that requires a full
	restart is needed, other methods such as notifications should be preferred.
*/
export const restartServer = async () => {
	await stopServer();
	await startServer();
};
