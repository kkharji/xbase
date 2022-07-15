import {
  Disposable, TextEditor, Uri, window, workspace, WorkspaceFolder, WorkspaceFoldersChangeEvent
} from "vscode";
import Logger from "./ui/logger";
import Server from "./server";
import { Runners } from "./types";
import FolderContext from "./folderContext";
import { isPathInsidePath, isSupportedProjectRoot, pathExists } from "./util";
import { dirname } from "path";
import * as commands from "./commands";
import Statusline from "./ui/statusline";

/**
 * Context for whole workspace. Holds array of contexts for each workspace folder
 * and the ExtensionContext
 * @credit mutli-workspace support @adam-fowler
 */
export class WorkspaceContext implements Disposable {
  public server: Server;
  public runners: Runners;
  public logger = new Logger();
  public folders: FolderContext[] = [];
  public currentFolder: FolderContext | null | undefined;
  public subscriptions: { dispose(): unknown }[] = [];
  public statusline = new Statusline();
  private observers: Set<WorkspaceFoldersObserver> = new Set();

  public static async init(): Promise<WorkspaceContext> {
    const server = await Server.connect();
    const runners = await server.request({ method: "get_runners" }) as Runners;
    return new WorkspaceContext(server, runners);
  }

  constructor(server: Server, runners: Runners) {
    const onChangeConfig = workspace.onDidChangeConfiguration(event => {
      console.log(event);
    });

    this.server = server;
    this.runners = runners;
    this.subscriptions = [
      this.server,
      this.logger,
      this.statusline,
      onChangeConfig
    ];
  };

  /**
   * Called whenever a folder is added to the workspace
   * @param folder folder being added
   */
  async addWorkspaceFolder(folder: WorkspaceFolder) {
    if (await isSupportedProjectRoot(folder.uri.fsPath))
      await this.addFolder(folder.uri, folder);

    if (this.getActiveWorkspaceFolder(window.activeTextEditor) === folder)
      await this.focusTextEditor(window.activeTextEditor);
  }

  /** Add workspace folders at initialisation */
  async addWorkspaceFolders() {
    // add workspace folders, already loaded
    if (workspace.workspaceFolders && workspace.workspaceFolders.length > 0) {
      for (const folder of workspace.workspaceFolders)
        await this.addWorkspaceFolder(folder);
    }
    // If we don't have a current selected folder Start up language server by firing focus event
    // on either null folder or the first folder if there is only one
    if (this.currentFolder === undefined) {
      if (this.folders.length === 1) {
        this.currentFolder = this.folders[0];
        await this.fireEvent(this.folders[0], FolderEvent.focus);
      } else {
        await this.fireEvent(null, FolderEvent.focus);
      }
    }
  }

  registerCommands() {
    commands.register(this);
  }

  async addFolder(folder: Uri, workspaceFolder: WorkspaceFolder): Promise<FolderContext | void> {
    try {
      const folderContext = await FolderContext.init(folder, workspaceFolder, this);
      this.folders.push(folderContext);

      await this.fireEvent(folderContext, FolderEvent.add);

      return folderContext;

    } catch (error) {
      console.error(error);
      window.showErrorMessage(`${error}`);
    }
  }

  /**
   * called when a folder is removed from workspace
   * @param folder folder being removed
   */
  async removeFolder(folder: WorkspaceFolder) {
    // find context with root folder
    const index = this.folders.findIndex(context => context.folder === folder);
    if (index === -1) {
      console.error(`Trying to delete folder ${folder} which has no record`);
      return;
    }
    const context = this.folders[index];
    // if current folder is this folder send unfocus event by setting
    // current folder to undefined
    if (this.currentFolder === context)
      this.focusFolder(null);

    // run observer functions in reverse order when removing
    const observersReversed = [...this.observers];
    observersReversed.reverse();
    for (const observer of observersReversed)
      await observer(context, FolderEvent.remove, this);

    context.dispose();
    // remove context
    this.folders.splice(index, 1);
  }

  private async getFolder(url: Uri): Promise<FolderContext | Uri | undefined> {
    // is editor document in any of the current FolderContexts
    const folder = this.folders.find(context => {
      return isPathInsidePath(url.fsPath, context.uri.fsPath);
    });

    if (folder) return folder;

    // if not search directory tree for 'Package.swift' files
    const workspaceFolder = workspace.getWorkspaceFolder(url);;
    if (!workspaceFolder) return;

    const workspacePath = workspaceFolder.uri.fsPath;
    let packagePath: string | undefined = undefined;
    let currentFolder = dirname(url.fsPath);
    // does Package.swift exist in this folder
    if (await pathExists(currentFolder, "Package.swift"))
      packagePath = currentFolder;

    // does Package.swift exist in any parent folders up to the root of the
    // workspace
    while (currentFolder !== workspacePath) {
      currentFolder = dirname(currentFolder);
      if (await isSupportedProjectRoot(currentFolder))
        packagePath = currentFolder;

    }
    if (packagePath) return Uri.file(packagePath);
  }


  /**
   * Add workspace folder event observer
   * @param fn observer function to be called when event occurs
   * @returns disposable object
   */
  observeFolders(fn: WorkspaceFoldersObserver): Disposable {
    this.observers.add(fn);
    return { dispose: () => this.observers.delete(fn) };
  }

  /**
   * Fire an event to all folder observers
   * @param folder folder to fire event for
   * @param event event type
   */
  async fireEvent(folder: FolderContext | null, event: FolderEvent) {
    for (const observer of this.observers)
      await observer(folder, event, this);
  }

  /**
   * set the focus folder
   * @param folderContext folder that has gained focus, you can have a null folder
   */
  async focusFolder(folderContext: FolderContext | null) {
    // null and undefined mean different things here. Undefined means nothing
    // has been setup, null means we want to send focus events but for a null
    // folder
    if (folderContext === this.currentFolder) return;

    // send unfocus event for previous folder observers
    if (this.currentFolder !== undefined)
      await this.fireEvent(this.currentFolder, FolderEvent.unfocus);

    this.currentFolder = folderContext;

    // send focus event to all observers
    await this.fireEvent(folderContext, FolderEvent.focus);
  }

  /** send unfocus event to current focussed folder and clear current folder */
  private async unfocusCurrentFolder() {
    // send unfocus event for previous folder observers
    if (this.currentFolder !== undefined)
      await this.fireEvent(this.currentFolder, FolderEvent.unfocus);

    this.currentFolder = undefined;
  }


  /** return workspace folder from text editor */
  private getActiveWorkspaceFolder(editor?: TextEditor): WorkspaceFolder | undefined {
    if (!editor || !editor.document) return;
    return workspace.getWorkspaceFolder(editor.document.uri);
  }


  /** set focus based on the file a TextEditor is editing */
  async focusTextEditor(editor?: TextEditor) {
    if (!editor || !editor.document || editor.document.uri.scheme !== "file")
      return;

    await this.focusUri(editor.document.uri);
  }

  /** set focus based on the file */
  async focusUri(uri: Uri) {
    const packageFolder = await this.getFolder(uri);
    if (packageFolder instanceof FolderContext) {
      await this.focusFolder(packageFolder);
    } else if (packageFolder instanceof Uri) {
      const workspaceFolder = workspace.getWorkspaceFolder(packageFolder);
      if (!workspaceFolder)
        return;

      await this.unfocusCurrentFolder();
      const folderContext = await this.addFolder(packageFolder, workspaceFolder);
      if (folderContext)
        await this.focusFolder(folderContext);
    } else {
      await this.focusFolder(null);
    }
  }

  /**
   * catch workspace folder changes and add or remove folders based on those changes
   * @param event workspace folder event
   */
  async onDidChangeWorkspaceFolders(event: WorkspaceFoldersChangeEvent) {
    for (const folder of event.added)
      await this.addWorkspaceFolder(folder);

    for (const folder of event.removed)
      await this.removeFolder(folder);
  }

  /** Setup the vscode event listeners to catch folder changes and active window changes */
  setupEventListeners() {
    // add event listener for when a workspace folder is added/removed
    const onWorkspaceChange = workspace.onDidChangeWorkspaceFolders(event => {
      if (this === undefined) {
        console.log("Trying to run onDidChangeWorkspaceFolders on deleted context");
        return;
      }
      this.onDidChangeWorkspaceFolders(event);
    });
    // add event listener for when the active edited text document changes
    const onDidChangeActiveWindow = window.onDidChangeActiveTextEditor(async editor => {
      if (this === undefined) {
        console.log("Trying to run onDidChangeWorkspaceFolders on deleted context");
        return;
      }
      await this.focusTextEditor(editor);
    });
    this.subscriptions.push(onWorkspaceChange, onDidChangeActiveWindow);
  }

  dispose() {
    // this.folders.forEach(f => f.dispose());
    this.subscriptions.forEach(item => item.dispose());
    this.server.dispose();
  }
}

/** Workspace Folder events */
export enum FolderEvent {
  // Workspace folder has been added
  add = "add",
  // Workspace folder has been removed
  remove = "remove",
  // Workspace folder has gained focus via a file inside the folder becoming the actively edited file
  focus = "focus",
  // Workspace folder loses focus because another workspace folder gained it
  unfocus = "unfocus",
}

/** Workspace Folder observer function */
export type WorkspaceFoldersObserver = (
  folder: FolderContext | null,
  operation: FolderEvent,
  workspace: WorkspaceContext
) => unknown;
