/* eslint-disable @typescript-eslint/no-unused-vars */
import { ExtensionContext } from "vscode";
import { WorkspaceContext } from "./workspaceContext";

/**
 * External API as exposed by the extension. Can be queried by other extensions
 * or by the integration test runner for VSCode extensions.
 */
export interface Api {
  workspaceContext: WorkspaceContext;
}

export async function activate(context: ExtensionContext): Promise<Api> {
  console.info("Activating XBase");

  const workspaceContext = await WorkspaceContext.init();

  workspaceContext.setupEventListeners();
  workspaceContext.addWorkspaceFolders();
  workspaceContext.registerCommands();


  context.subscriptions.push(workspaceContext);

  return { workspaceContext };
}

export function deactivate() {
  return;
}
