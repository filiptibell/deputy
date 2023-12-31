/* eslint-disable @typescript-eslint/naming-convention */

import * as vscode from "vscode";
import axios from "axios";

import { getExtensionContext } from "../extension";

const GITHUB_AUTH_TOKEN_STORAGE_KEY = "auth.github.token";

// https://docs.github.com/en/rest/overview/authenticating-to-the-rest-api
const validate = async (token: string): Promise<boolean> => {
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

/**
	Prompts the user for a GitHub Personal Access Token.

	If provided, this token will be validated against the GitHub
	authorization API and permanently stored in the global storage.

	Any token returned from this function is guaranteed to currently be valid.
*/
export const prompt = async (
	skipNotification?: true
): Promise<string | null> => {
	const context = getExtensionContext();

	if (skipNotification !== true) {
		const result = await vscode.window.showInformationMessage(
			"The GitHub API rate limit has been reached." +
				"\nSome functionality will be disabled until authenticated.",
			"Set Personal Access Token"
		);
		if (result !== "Set Personal Access Token") {
			return null;
		}
	}

	let prompt = "Enter a token";
	while (true) {
		const token = await vscode.window.showInputBox({
			prompt,
			title: "GitHub Personal Access Token",
			password: true,
			ignoreFocusOut: true,
		});
		if (token !== undefined) {
			if (token !== "" && (await validate(token))) {
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

	return null;
};

/**
	Retrieves any currently stored GitHub Personal Access Token from global storage.

	This will perform validation against the GitHub authorization API, and if the
	stored token is no longer valid, it will be removed, and `undefined` is returned.

	Any token returned from this function is guaranteed to currently be valid.
*/
export const get = async (): Promise<string | undefined> => {
	const context = getExtensionContext();

	const token = context.globalState.get(GITHUB_AUTH_TOKEN_STORAGE_KEY);
	if (typeof token === "string" && token.length > 0) {
		if (await validate(token)) {
			return token;
		} else {
			await context.globalState.update(
				GITHUB_AUTH_TOKEN_STORAGE_KEY,
				undefined
			);
		}
	}

	return undefined;
};

/**
	Removes any currently stored GitHub Personal Access Token from global storage.

	Returns `true` if the token was removed from global storage, `false` if it did not exist.
*/
export const reset = async (): Promise<boolean> => {
	const context = getExtensionContext();

	if (!!context.globalState.get(GITHUB_AUTH_TOKEN_STORAGE_KEY)) {
		await context.globalState.update(
			GITHUB_AUTH_TOKEN_STORAGE_KEY,
			undefined
		);
		return true;
	} else {
		return false;
	}
};
