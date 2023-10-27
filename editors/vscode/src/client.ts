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

import { getExtensionContext } from "./extension";

import auth from "./auth";
import requests from "./requests";
import util from "./util";

let client: LanguageClient | undefined;
let outputChannel: vscode.OutputChannel;

/**
	Starts the language server.

	Will throw an error if the language server has already been started.
*/
export const startServer = async () => {
	if (client !== undefined) {
		throw new Error("Language server has already been started");
	}

	const context = getExtensionContext();

	// Create persistent output channel if one does not exist

	if (outputChannel === undefined) {
		outputChannel = vscode.window.createOutputChannel(
			"Tooling Language Server"
		);
	}

	// Retrieve and validate stored authentication, if any

	const githubAuthToken = await auth.github.get();

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

	const command = (await util.fs.fileExists(exeRelease))
		? exeRelease.fsPath
		: (await util.fs.fileExists(exeDebug))
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

	options.env["GITHUB_TOKEN"] = githubAuthToken;

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
		documentSelector: [
			{ scheme: "file", language: "toml" },
			{ scheme: "file", language: "json" },
		],
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
	client.onRequest(
		requests.rateLimit.RATE_LIMIT_METHOD,
		requests.rateLimit.handleRateLimitRequest
	);
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
