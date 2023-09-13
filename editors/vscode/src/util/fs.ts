import * as vscode from "vscode";

export const fileExists = async (path: vscode.Uri): Promise<boolean> => {
	try {
		let stat = await vscode.workspace.fs.stat(path);
		return stat.type === vscode.FileType.File;
	} catch {
		return false;
	}
};
