import * as path from "path";
import { Disposable, Uri, WorkspaceFolder } from "vscode";
import Broadcast from "./broadcast";
import { ProjectInfo } from "./types";
import { projectName } from "./util";
import { WorkspaceContext } from "./workspaceContext";

export default class FolderContext implements Disposable {
  public projectInfo: ProjectInfo = { watchlist: [], targets: {} };
  public subscriptions: Disposable[] = [];
  private constructor(
    public ctx: WorkspaceContext,
    public uri: Uri,
    public folder: WorkspaceFolder,
  ) { }

  /**
   * Initialize new FolderContext.
   * Register Folder Root using `WorkspaceCOntext.server.register`
   */
  static async init(
    uri: Uri, folder: WorkspaceFolder, ctx: WorkspaceContext
  ): Promise<FolderContext> {
    const name = projectName(uri.fsPath);
    const registering = `[${name}] Registering`;
    const folderCtx = new FolderContext(ctx, uri, folder);

    ctx.statusline.update({ content: registering });
    console.log(registering);

    folderCtx.subscriptions.push(
      await ctx.server.register(uri.fsPath)
        .then(address => Broadcast.connect(folderCtx, address, ctx))
        .catch(error => {
          throw Error(`[${name}] Failed to Initialize: ${error}`);
        }));

    ctx.statusline.setDefault();
    console.log(`[${name}] Registered`);

    return folderCtx;
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
    this.subscriptions.map(v => v.dispose());
  }

}
