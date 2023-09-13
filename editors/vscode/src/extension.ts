/* eslint-disable @typescript-eslint/naming-convention */

import * as vscode from "vscode";

import { setExtensionContext, startServer, stopServer } from "./client";

export async function activate(context: vscode.ExtensionContext) {
	setExtensionContext(context);
	await startServer();
}

export async function deactivate() {
	await stopServer();
}
