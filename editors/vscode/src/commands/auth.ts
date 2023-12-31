import * as client from "../client";

import auth from "../auth";

export const promptAuthForGitHub = async (args: {}) => {
	await auth.github.prompt(true);
	await client.restartServer();
};

export const resetAuthForGitHub = async (args: {}) => {
	await auth.github.reset();
	await client.restartServer();
};

export default {
	promptAuthForGitHub,
	resetAuthForGitHub,
};
