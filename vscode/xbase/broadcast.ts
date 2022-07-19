import { filter, map, pipe, split, toAsync } from "iter-ops";
import net from "net";
import { Disposable, window, commands } from "vscode";
import { Message, ContentLevel, TaskKind, TaskStatus } from "./types";
import Logger from "./ui/logger";
import Statusline from "./ui/statusline";
import configuration from "./config";
import { WorkspaceContext } from "./workspaceContext";
import SourcekitLsp from "./sourcekit-lsp";
import FolderContext from "./folderContext";

interface CurrentTask {
  prefix: { processing: string, done: string },
  target: string,
  kind: TaskKind,
  status: TaskStatus,
}

export default class Broadcast implements Disposable {
  public name: string;
  public folderCtx: FolderContext;
  public socket: net.Socket;
  private logger: Logger;
  private statusline: Statusline;
  private currentTask?: CurrentTask;
  private sourcekit: SourcekitLsp;

  private constructor(
    folder: FolderContext, socket: net.Socket, ctx: WorkspaceContext
  ) {
    this.folderCtx = folder;
    this.name = folder.name.charAt(0).toUpperCase() + folder.name.slice(1);
    this.socket = socket;
    this.logger = ctx.logger;
    this.statusline = ctx.statusline;
    this.sourcekit = ctx.sourcekit;
  }

  public static async connect(
    folder: FolderContext,
    address: string,
    ctx: WorkspaceContext
  ): Promise<Broadcast> {
    return new Promise((resolve, reject) => {
      const socket = net.createConnection(address, () => {
        const broadcast = new Broadcast(folder, socket, ctx);
        socket.on("data", async buffer => {
          for await (const message of Broadcast.get_messages(buffer))
            await broadcast.handleMessage(message);
        });
        socket.write(`${process.pid}\n`);
        resolve(broadcast);
      });
      socket.on("error", err => {
        reject(Error(`Failed to connect to XBase Broadcast: ${err}`));
      });
    });
  }

  private static get_messages(buffer: Buffer): AsyncIterable<Message> {
    return pipe(
      toAsync(buffer),
      split(a => a === 10),
      map(m => Buffer.from(m)),
      filter(m => m.length > 1),
      map(m => JSON.parse(m.toString()) as Message)
    );
  }

  private async handleMessage(message: Message) {
    switch (message.type) {
      case "Notify": {
        const { content, level } = message.args;
        await this.notify(content, level);
        break;
      }
      case "Log": {
        const { content, level } = message.args;
        // TODO: check for current log level
        if (!levelShouldIgnore(level))
          this.logger.append(content, level);
        break;
      }
      case "OpenLogger":
        if (configuration.ui.openLoggerOnError)
          this.logger.toggle();
        break;
      case "SetCurrentTask":
        this.setTask(message.args.kind, message.args.target, message.args.status);
        break;
      case "UpdateCurrentTask":
        this.updateCurrentTask(message.args.content, message.args.level);
        break;
      case "FinishCurrentTask":
        await this.finishTask(message.args.status);
        break;
      case "ReloadLspServer":
        await this.sourcekit.restartClient(this.folderCtx.uri);
        break;
      case "SetState":
        switch (message.args.key) {
          case "runners":
            console.log("Runners are set");
            this.folderCtx.ctx.runners = message.args.value;
            break;
          case "projectInfo":
            console.log("projectInfo is set");
            this.folderCtx.projectInfo = message.args.value;
            break;
        }
        break;
    }
  }

  private setTask(kind: TaskKind, target: string, status: TaskStatus) {
    const prefix = TaskKind.prefix(kind)!;
    this.currentTask = { target, kind, status, prefix };
    this.statusline.update({
      content: `[${target}] ${prefix.processing}`,
      icon: TaskKind.isRun(kind) ? "$(code)" : undefined
    });
  }

  private updateCurrentTask(content: string, level: ContentLevel) {
    if (this.currentTask === undefined) {
      console.warn("trying to update task that no longer exists!");
      return;
    };

    if (levelShouldIgnore(level))
      return;

    const { target, prefix, kind } = this.currentTask;

    this.logger.append(content, level);

    content = content.replace(`[${this.currentTask.target}]`, "");

    this.statusline.update({
      content: `[${target}] ${prefix.processing}: ${content}`,
      icon: TaskKind.isRun(kind) ? "$(code)" : undefined,
      level
    });
  }

  private async finishTask(status: TaskStatus) {
    if (this.currentTask === undefined) {
      console.warn("trying to finish task that no longer exists!");
      return;
    }

    const { target, prefix, kind } = { ...this.currentTask };
    const taskFailed = (status === "Failed");
    this.currentTask = undefined;

    const level = taskFailed ? "Error" : "Info";
    const content = TaskKind.isRun(kind)
      ? `[${target}] Device disconnected`
      : (taskFailed
        ? `[${target}] ${prefix.processing} Failed`
        : `[${target}] ${prefix.done}`);

    this.logger.append(content, level);

    this.statusline.set({
      icon: taskFailed ? "$(error)" : "$(pass)",
      content,
      level,
      is_success: taskFailed ? false : true
    });


    if (!taskFailed) {
      await (new Promise(resolve => setTimeout(resolve, 3000)));
      this.statusline.setDefault();
    }
  }

  private async notify(msg: string, level: ContentLevel) {
    switch (level) {
      case "Info":
        window.showInformationMessage(msg);
        if (msg.trim().endsWith("Registered")) {
          setTimeout(async () => {
            console.log("Clearing ...");
            await commands.executeCommand("notifications.clearAll");
          }, 3500);
        }
        break;
      case "Warn":
        window.showWarningMessage(msg);
        break;
      case "Error":
        window.showErrorMessage(msg);
        break;
    }
  }

  dispose() {
    this.socket.pause();
    this.socket.end();
    this.socket.destroy();
  }
}

const levelShouldIgnore = (level: ContentLevel) =>
  (level === "Debug" || level === "Trace");
