import * as server from "../server";

import auth from "../auth";

export const promptAuthForGitHub = async (args: {}) => {
	await auth.github.prompt(true);
	await server.restart();
};

export const resetAuthForGitHub = async (args: {}) => {
	await auth.github.reset();
	await server.restart();
};

export default {
	promptAuthForGitHub,
	resetAuthForGitHub,
};
