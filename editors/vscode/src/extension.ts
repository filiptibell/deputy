import * as vscode from "vscode";

import * as server from "./server";

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

	await server.start();
}

export async function deactivate() {
	await server.stop();
}
