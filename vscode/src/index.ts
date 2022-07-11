/* eslint-disable @typescript-eslint/no-unused-vars */
import type { ExtensionContext } from "vscode";
import { window, workspace } from "vscode";
import * as util from "./util";
import XBaseServer from "./server";
import XBaseOutputChannel from "./ui/output";
import XBaseCommands from "./commands";
import XBaseState from "./state";

let server: XBaseServer;

export async function activate(_context: ExtensionContext) {
  console.info("Activating XBase");

  let root: string;
  const channel = new XBaseOutputChannel();
  const connect = await XBaseServer.connect(channel);

  if (workspace.workspaceFolders && (workspace.workspaceFolders.length > 0)) {
    root = workspace.workspaceFolders[0].uri.fsPath;
  }
  else if (workspace.workspaceFolders) {
    root = workspace.workspaceFolders[0].uri.fsPath;
    console.warn(`Found multiple roots, using ${root}`);
  }
  else {
    console.error("No Workspace Opened, ignoring ..");
    return;
  }

  if (connect.isErr()) {
    console.error(`Fail to activate XBase: ${connect.unwrapErr()}`);
    return;
  }
  server = connect.unwrap();

  // TODO: Setup Editor Status
  const register = await server.register(root);

  if (register.isErr()) {
    console.error(`Fail to register workspaceFolder: ${register.unwrapErr()}`);
    return;
  }

  const name = util.projectName(root);

  window.showInformationMessage(`[${name}] Registered`);

  const init_state = await XBaseState.init(server);

  if (init_state.isErr()) {
    const msg = `Failed to initialize state ${init_state.unwrapErr()}`;
    window.showErrorMessage(msg);
    console.error(msg);
  }

  const state = init_state.unwrap();

  new XBaseCommands(server, state);

}

export function deactivate(_context: ExtensionContext) {
  // TODO: Drop all roots
  server.socket.end();
}
