import * as vscode from "vscode";

import authCommands from "./auth";

const ALL_COMMANDS = {
	...authCommands,
};

// https://stackoverflow.com/questions/51851677/how-to-get-argument-types-from-function-in-typescript
type ArgumentTypes<F extends Function> = F extends (...args: infer A) => any
	? A
	: never;

type Commands = typeof ALL_COMMANDS;
type CommandName = keyof Commands;

/**
	Creates a markdown-friendly and clickable link for running a command.

	This will run the command without any arguments.
*/
export const getCommandLink = <N extends CommandName>(commandName: N) => {
	return vscode.Uri.parse(`command:tooling-language-server.${commandName}`);
};

/**
	Creates a markdown-friendly and clickable link for running a command.

	This will run the command with the given arguments.
*/
export const getCommandLinkWithArgs = <N extends CommandName>(
	commandName: N,
	...args: ArgumentTypes<Commands[N]>
) => {
	const encoded = encodeURIComponent(JSON.stringify(args));
	return vscode.Uri.parse(`${getCommandLink(commandName)}?${encoded}`);
};

/**
	Command provider class that will register all known commands for the extension.

	Implements `vscode.Disposable` to clean up any registered commands.
*/
export class CommandProvider implements vscode.Disposable {
	private disposed: boolean = false;
	private disposables: vscode.Disposable[];

	constructor() {
		const disposables = [];
		for (const [commandName, commandHandler] of Object.entries(
			ALL_COMMANDS
		)) {
			const commandIdentifier = `tooling-language-server.${commandName}`;
			disposables.push(
				vscode.commands.registerCommand(
					commandIdentifier,
					(...args) => {
						// NOTE: We need to cast `handler` to `any` here because of typescript error 2556:
						// "A spread argument must either have a tuple type or be passed to a rest parameter"
						// Correct types should always be enforced by the exported command link functions
						const untyped = commandHandler as any;
						return untyped(...args);
					}
				)
			);
		}
		this.disposables = disposables;
	}

	dispose() {
		if (this.disposed !== true) {
			this.disposed = true;
			for (const disposable of this.disposables) {
				disposable.dispose();
			}
		}
	}

	/**
		Runs the command with the given args.

		See `getCommandLink` and / or `getCommandLinkWithArgs`
		to let a user run a command by clicking a markdown link.
	*/
	run<Name extends CommandName>(
		commandName: Name,
		...args: ArgumentTypes<Commands[Name]>
	): ReturnType<Commands[Name]> {
		// NOTE: We need to cast `handler` to `any` here
		// because of typescript error 2556, see above.
		const handler = ALL_COMMANDS[commandName];
		const untyped = handler as any;
		return untyped(...args);
	}
}
