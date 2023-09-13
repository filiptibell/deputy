import * as vscode from "vscode";

import * as client from "./client";

import { CommandProvider } from "./commands";

let context: vscode.ExtensionContext;

export function getExtensionContext(): vscode.ExtensionContext {
	if (context !== undefined) {
		return context;
	} else {
		throw new Error("Missing extension context");
	}
}

export async function activate(ctx: vscode.ExtensionContext) {
	context = ctx;

	ctx.subscriptions.push(new CommandProvider());

	await client.startServer();
}

export async function deactivate() {
	await client.stopServer();
}
