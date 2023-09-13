/* eslint-disable @typescript-eslint/naming-convention */

import * as vscode from "vscode";
import axios from "axios";

const GITHUB_AUTH_TOKEN_STORAGE_KEY = "auth.github.token";

// https://docs.github.com/en/rest/overview/authenticating-to-the-rest-api
const validateAuthForGitHub = async (
	context: vscode.ExtensionContext,
	token: string
): Promise<boolean> => {
	return new Promise((resolve) => {
		axios
			.get("https://api.github.com/octocat", {
				headers: {
					Authorization: `Bearer ${token}`,
					"X-GitHub-Api-Version": "2022-11-28",
				},
			})
			.then((_) => {
				resolve(true);
			})
			.catch((e) => {
				resolve(false);
			});
	});
};

export const promptAuthForGitHub = async (
	context: vscode.ExtensionContext
): Promise<string | null> => {
	const result = await vscode.window.showInformationMessage(
		"The GitHub API rate limit has been reached." +
			"\nSome functionality will be disabled until authenticated.",
		"Set Personal Access Token"
	);
	if (result === "Set Personal Access Token") {
		let prompt = "Enter a token";
		while (true) {
			const token = await vscode.window.showInputBox({
				prompt,
				title: "GitHub Personal Access Token",
				password: true,
				ignoreFocusOut: true,
			});
			if (token !== undefined) {
				if (
					token !== "" &&
					(await validateAuthForGitHub(context, token))
				) {
					await context.globalState.update(
						GITHUB_AUTH_TOKEN_STORAGE_KEY,
						token
					);
					return token;
				} else {
					prompt = "Token is not valid. Enter a new token";
					continue;
				}
			} else {
				break;
			}
		}
	}
	return null;
};

export const getAuthForGitHub = (
	context: vscode.ExtensionContext
): string | undefined => {
	const token = context.globalState.get(GITHUB_AUTH_TOKEN_STORAGE_KEY);
	if (typeof token === "string" && token.length > 0) {
		return token;
	} else {
		return undefined;
	}
};

export const resetAuthForGitHub = (
	context: vscode.ExtensionContext
): boolean => {
	if (!!context.globalState.get(GITHUB_AUTH_TOKEN_STORAGE_KEY)) {
		context.globalState.update(GITHUB_AUTH_TOKEN_STORAGE_KEY, undefined);
		return true;
	} else {
		return false;
	}
};
