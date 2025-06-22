/* eslint-disable @typescript-eslint/naming-convention */

import * as vscode from "vscode";
import which from "which";

import {
	Executable,
	LanguageClient,
	LanguageClientOptions,
	ServerOptions,
} from "vscode-languageclient/node";

import auth from "../auth";
import requests from "../requests";
import { getExtensionContext } from "../extension";
import { Downloader } from "./downloader";
import { BINARY_NAME, DISPLAY_NAME } from "./constants";

let client: LanguageClient | undefined;
let outputChannel: vscode.OutputChannel;

/**
	Starts the language server.

	Will throw an error if the language server has already been started.
*/
export const start = async () => {
	if (client !== undefined) {
		throw new Error("Language server has already been started");
	}

	const context = getExtensionContext();

	// Create persistent output channel if one does not exist

	if (outputChannel === undefined) {
		outputChannel = vscode.window.createOutputChannel(DISPLAY_NAME);
	}

	// Retrieve and validate stored authentication, if any

	const githubAuthToken = await auth.github.get();

	// Check if we have the server binary on PATH, download it if not

	let resolved = await which(BINARY_NAME, { nothrow: true });
	if (!resolved) {
		const downloader = new Downloader(context, outputChannel);

		await vscode.window.withProgress(
			{
				location: vscode.ProgressLocation.Window,
				title: `Downloading ${DISPLAY_NAME}...`,
			},
			async () => {
				await downloader.download();
			},
		);

		resolved = downloader.path();
	}

	// Create args for language server

	const server: Executable = {
		command: resolved,
		options: {
			env: { PATH: process.env.PATH, GITHUB_TOKEN: githubAuthToken },
		},
		args: ["serve"],
	};

	// Create language server & client config

	const serverOptions: ServerOptions = {
		run: server,
		debug: server,
	};

	const clientOptions: LanguageClientOptions = {
		stdioEncoding: "utf8",
		documentSelector: [{ scheme: "file", language: "zap" }],
		outputChannel,
	};

	// Start language server & client

	outputChannel.appendLine("Starting language server");

	client = new LanguageClient(
		BINARY_NAME,
		DISPLAY_NAME,
		serverOptions,
		clientOptions,
	);

	client.start();

	// Listen for custom requests from server

	client.onRequest(
		requests.rateLimit.RATE_LIMIT_METHOD,
		requests.rateLimit.handleRateLimitRequest,
	);
};

/**
	Stops the language server.

	Returns `true` if stopped, `false` if the language server was not running.
*/
export const stop = async (): Promise<boolean> => {
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
export const restart = async () => {
	await stop();
	await start();
};
