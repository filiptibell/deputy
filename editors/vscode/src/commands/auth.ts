import * as client from "../client";

import auth from "../auth";

export const resetAuthForGitHub = async (args: {}) => {
	await auth.github.reset();
	await client.restartServer();
};

export default {
	resetAuthForGitHub,
};
