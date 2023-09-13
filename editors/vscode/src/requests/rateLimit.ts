import auth from "../auth";

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
	request: RateLimitRequest
): Promise<RateLimitResponse> => {
	let response: RateLimitResponse = {
		kind: request.kind,
		value: null,
	};

	if (request.kind === "GitHub") {
		const token = await auth.github.prompt();
		if (token) {
			response.value = token;
		}
	}

	return response;
};
