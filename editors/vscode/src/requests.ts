import * as vscode from "vscode";

import { promptAuthForGitHub } from "./auth";

export const RATE_LIMIT_METHOD = "$/internal_request/rate_limit";

export type RateLimitKind = "GitHub";

export type RateLimitRequest = {
	kind: RateLimitKind;
	value: any;
};

export type RateLimitResponse = {
	kind: RateLimitKind;
	value: any;
};

export const handleRateLimitRequest = async (
	context: vscode.ExtensionContext,
	request: RateLimitRequest
): Promise<RateLimitResponse> => {
	let response: RateLimitResponse = {
		kind: request.kind,
		value: null,
	};

	if (request.kind === "GitHub") {
		const auth = await promptAuthForGitHub(context);
		if (auth) {
			response.value = auth;
		}
	}

	return response;
};
