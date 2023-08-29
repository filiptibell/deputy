import * as vscode from "vscode";
import * as path from "path";

import {
	LanguageClient,
	LanguageClientOptions,
	ServerOptions,
	TransportKind,
} from "vscode-languageclient/node";

let client: LanguageClient;

export async function activate(context: vscode.ExtensionContext) {
	//
}

export function deactivate() {
	if (client) {
		client.stop();
	}
}
