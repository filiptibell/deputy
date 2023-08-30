/* eslint-disable @typescript-eslint/naming-convention */

import * as vscode from "vscode";
import axios from "axios";

// https://docs.github.com/en/rest/overview/authenticating-to-the-rest-api
export const validateAuthForGitHub = async (
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

export const promptAuthForGitHub = async (): Promise<string | null> => {
	const result = await vscode.window.showInformationMessage(
		"The GitHub API rate limit has been reached." +
			"\nFunctionality will be disabled until authenticated.",
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
				if (token !== "" && (await validateAuthForGitHub(token))) {
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
