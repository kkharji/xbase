import * as path from "path";
import { Disposable, Uri, window, WorkspaceFolder } from "vscode";
import Broadcast from "./broadcast";
import { projectName } from "./util";
import { WorkspaceContext } from "./workspaceContext";

export default class FolderContext implements Disposable {
  private constructor(
    public uri: Uri,
    public folder: WorkspaceFolder,
    private broadcast: Broadcast
  ) { }

  /**
   * Initialize new FolderContext.
   * Register Folder Root using `WorkspaceCOntext.server.register`
   */
  static async init(
    uri: Uri, folder: WorkspaceFolder, ctx: WorkspaceContext
  ): Promise<FolderContext> {
    const name = projectName(uri.fsPath);
    const statusItemText = `[${name}] Registering`;
    console.log(statusItemText);

    // ctx.statusItem.start(statusItemText);
    const broadcast = await ctx.server.register(uri.fsPath)
      .then(address => Broadcast.connect(address, ctx.outputChannel))
      .catch(error => {
        throw Error(`[${name}] Failed to Initialize: ${error}`);
      });
    const registered = `[${name}] Registered`;
    // ctx.statusItem.end(statusItemText);
    window.showInformationMessage(registered);
    console.log(registered);

    return new FolderContext(uri, folder, broadcast);
  }

  get relativePath(): string {
    return path.relative(this.folder.uri.fsPath, this.uri.fsPath);
  }

  get name(): string {
    const relativePath = this.relativePath;
    if (relativePath.length === 0)
      return this.folder.name;
    else
      return `${this.folder.name}/${this.relativePath}`;
  }

  get isRootFolder(): boolean {
    return this.folder.uri === this.uri;
  }

  dispose() {
    this.broadcast.dispose();
  }

}
