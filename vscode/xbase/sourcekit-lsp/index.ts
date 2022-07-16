// credit: inspired by @adam-fowler and @vknabel implementation

import * as vs from "vscode";
import * as lc from "vscode-languageclient/node";
import configuration from "../config";
import { isPathInsidePath, projectName } from "../util";
import { FolderEvent, WorkspaceContext } from "../workspaceContext";

export default class SourcekitLsp implements vs.Disposable {

  /* Statics */
  private static id = "sourcekit-lsp";
  private static title = "SourceKit Language Server";
  private static revealOutputChannelOn = lc.RevealOutputChannelOn.Never;
  public static documentSelector = [
    { scheme: "file", language: "swift" },
    { scheme: "untitled", language: "swift" },
    { scheme: "file", language: "c" },
    { scheme: "untitled", language: "c" },
    { scheme: "file", language: "cpp" },
    { scheme: "untitled", language: "cpp" },
    { scheme: "file", language: "objective-c" },
    { scheme: "untitled", language: "objective-c" },
    { scheme: "file", language: "objective-cpp" },
    { scheme: "untitled", language: "objective-cpp" },
  ];

  public workspaceContext: WorkspaceContext;

  /* undefined = not setup */
  /* null = the process of restarting */
  private languageClient?: lc.LanguageClient | null = undefined;
  private cancellationToken?: lc.CancellationTokenSource = undefined;

  /* Promises */
  private restartedPromise?: Promise<void>;
  private clientReadyPromise?: Promise<void>;

  private currentFolderUri?: vs.Uri = undefined;
  private documentSymbolWatcher?: DocumentSymbolWatcher = undefined;

  private waitingOnRestartCount: number;

  private subscriptions: { dispose(): unknown }[] = [];
  private subFolderWorkspaces: vs.Uri[] = [];

  private get serverOptions(): lc.ServerOptions {
    const { path: command, arguments: args } = configuration.lsp;
    const options = { env: { ...process.env, }, };

    return { command, args, options };
  }

  private clientOptions(uri?: vs.Uri): lc.LanguageClientOptions {
    const { documentSelector, revealOutputChannelOn } = SourcekitLsp;
    const workspaceFolder = uri ? { uri, name: projectName(uri.fsPath), index: 0, } : undefined;
    return {
      documentSelector,
      workspaceFolder,
      revealOutputChannelOn,
      middleware: {
        provideDocumentSymbols: async (document, token, next) => {
          const result = await next(document, token);
          const documentSymbols = result as vs.DocumentSymbol[];
          if (this.documentSymbolWatcher && documentSymbols)
            this.documentSymbolWatcher(document, documentSymbols);
          return result;
        },
      },
    };
  }

  constructor(workspaceContext: WorkspaceContext) {
    this.waitingOnRestartCount = 0;
    this.cancellationToken = new lc.CancellationTokenSource();
    this.workspaceContext = workspaceContext;
    this.subscriptions.push(this.onFolderSwtich());
    this.subscriptions.push(this.onConfigUpdate());
  }

  private async setupFolder(uri?: vs.Uri, forceRestart = false) {
    if (this.languageClient === undefined) {
      this.currentFolderUri = uri;
      this.restartedPromise = this.setupClient(uri);
    } else {
      // skip if not uri or currentWorkspaceFolder === uri and not fource
      const skip = uri === undefined || (this.currentFolderUri === uri && !forceRestart);
      if (!skip) this.restartClient(uri);
    }
  }

  private setupClient(uri?: vs.Uri, restart = false): Promise<void> {
    const { logger } = this.workspaceContext;
    const { id, title } = SourcekitLsp;
    const [serverOptions, clientOptions] = [this.serverOptions, this.clientOptions(uri)];
    const client = new lc.LanguageClient(id, title, serverOptions, clientOptions);
    const wsFolderName = client.clientOptions.workspaceFolder?.name;

    logger.append(`[${wsFolderName || "XBase"}] SourceKit-LSP ${restart ? "restarted" : "initialized"}`);

    client.onDidChangeState(e =>
      (e.oldState === 3 && e.newState === 2) && this.addSubFolderWorkspaces(client)
    );

    this.clientReadyPromise = client.start();
    this.languageClient = client;
    this.cancellationToken = new lc.CancellationTokenSource();
    return this.clientReadyPromise;
  }

  public async restartClient(uri?: vs.Uri) {
    const { restartedPromise, workspaceContext: { logger }, languageClient: client } = this;

    // Count number of setLanguageClientFolder calls waiting on startedPromise
    this.waitingOnRestartCount += 1;

    // if in the middle of a restart then we have to wait until that restart has finished
    if (restartedPromise) try { await restartedPromise; } catch (error) { /* ignore error*/ }

    this.waitingOnRestartCount -= 1;

    // Only continue if no more calls are waiting on startedPromise
    if (this.waitingOnRestartCount !== 0) return;

    // language client is set to null while it is in the process of restarting
    this.languageClient = null;
    this.currentFolderUri = uri;

    if (client) {
      this.cancellationToken?.cancel();
      this.cancellationToken?.dispose();
      this.restartedPromise = client.stop().then(
        async () => await this.setupClient(uri, true),
        err => logger.append(`${err}`, "Error"));
    }
  }


  private async addSubFolderWorkspaces(client: lc.LanguageClient) {
    const didChangeNotifcation = lc.DidChangeWorkspaceFoldersNotification.type;
    for (const uri of this.subFolderWorkspaces) {
      client.sendNotification(didChangeNotifcation, {
        event: {
          added: [{
            uri: client.code2ProtocolConverter.asUri(uri),
            name: projectName(uri.fsPath),
          }], removed: []
        },
      });
    }
  }


  /** Events */

  // Stop/Start server for folder switch
  private onFolderSwtich(): { dispose(): unknown; } {
    return this.workspaceContext.observeFolders(
      async (folderContext, event) => {
        const [isFocus, isAdd] = [event === FolderEvent.focus, event === FolderEvent.add];
        const folderUri = folderContext?.folder.uri;
        const currFilePath = vs.window.activeTextEditor?.document.uri.fsPath;
        if (isFocus || (isAdd
          && folderUri
          && currFilePath
          && isPathInsidePath(currFilePath, folderUri.fsPath)))
          await this.setupFolder(folderUri);
      }
    );
  }

  /// Listen to sourcekit-lsp configuration update
  private onConfigUpdate(): vs.Disposable {
    const onRestartMessage = "XBase: Changing SourcekitLsp config requires a restarted.";

    return vs.workspace.onDidChangeConfiguration(event => {
      const isPathChange = event.affectsConfiguration("xbase.sourcekit-lsp.path");
      const isArgsChange = event.affectsConfiguration("xbase.sourcekit-lsp.arguments");
      if (isPathChange || isArgsChange) {
        vs.window.showInformationMessage(onRestartMessage, "Ok").then(async (v) => {
          return v === "Ok" && await this.setupFolder(this.currentFolderUri, true);
        });
      }
    });
  }


  dispose() {
    this.cancellationToken?.cancel();
    this.cancellationToken?.dispose();
    this.subscriptions.forEach(item => item.dispose());
    this.languageClient?.stop();
  }
}
type DocumentSymbolWatcher = (
  document: vs.TextDocument,
  symbols: vs.DocumentSymbol[] | null | undefined
) => void;
