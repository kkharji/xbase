import { filter, map, pipe, split, toAsync } from "iter-ops";
import net from "net";
import { Disposable, window } from "vscode";
import { Message, ContentLevel, TaskKind, TaskStatus } from "./types";
import OutputChannel from "./ui/outputChannel";
import Statusline from "./ui/statusline";

interface CurrentTask {
  prefix: { processing: string, done: string },
  target: string,
  kind: TaskKind,
  status: TaskStatus,
}

export default class Broadcast implements Disposable {
  public name: string;
  public socket: net.Socket;
  private output: OutputChannel;
  private statusline: Statusline;
  private currentTask?: CurrentTask;

  private constructor(
    name: string, socket: net.Socket, output: OutputChannel, statusbar: Statusline
  ) {
    this.name = name.charAt(0).toUpperCase() + name.slice(1);
    this.socket = socket;
    this.output = output;
    this.statusline = statusbar;
  }

  public static async connect(
    name: string, address: string, logger: OutputChannel, statusbar: Statusline
  ): Promise<Broadcast> {
    return new Promise((resolve, reject) => {
      const socket = net.createConnection(address, () => {
        const broadcast = new Broadcast(name, socket, logger, statusbar);
        socket.on("data", async buffer => {
          for await (const message of Broadcast.get_messages(buffer))
            await broadcast.handleMessage(message);
        });
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
        if (level !== ContentLevel.Debug && level !== ContentLevel.Trace)
          this.output.append(content, level);
        break;
      }
      case "OpenLogger":
        this.output.show();
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
        // TODO: Implement
        break;
    }
  }

  private setTask(kind: TaskKind, target: string, status: TaskStatus) {
    const prefix = TaskKind.prefix(kind);
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

    if (!(level === ContentLevel.Debug || level === ContentLevel.Trace)) {
      const { target, prefix, kind } = this.currentTask;
      content = content.replace(`[${this.currentTask.target}]`, "");

      this.statusline.update({
        content: `[${target}] ${prefix.processing}: ${content}`,
        icon: TaskKind.isRun(kind) ? "$(code)" : undefined,
        level
      });
    }

    this.output.append(content, level);
  }

  private async finishTask(status: TaskStatus) {
    if (this.currentTask === undefined) {
      console.warn("trying to finish task that no longer exists!");
      return;
    }

    const { target, prefix, kind } = { ...this.currentTask };
    const taskFailed = (status === TaskStatus.Failed);
    this.currentTask = undefined;

    const level = taskFailed ? ContentLevel.Error : ContentLevel.Info;
    const content = TaskKind.isRun(kind)
      ? `[${target}] Device disconnected`
      : (taskFailed
        ? `[${target}] ${prefix.processing} Failed`
        : `[${target}] ${prefix.done}`);

    this.output.append(content, level);

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
        await window.showInformationMessage(msg);
        break;
      case "Warn":
        await window.showWarningMessage(msg);
        break;
      case "Error":
        await window.showErrorMessage(msg);
        break;
    }
  }

  dispose() {
    this.socket.pause();
    this.socket.end();
    this.socket.destroy();
  }
}
