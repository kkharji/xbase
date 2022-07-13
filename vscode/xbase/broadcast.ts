import { filter, map, pipe, split, toAsync } from "iter-ops";
import net from "net";
import { Disposable, window } from "vscode";
import { Message, MessageLevel, Task } from "./types";
import OutputChannel from "./ui/outputChannel";

export default class Broadcast implements Disposable {
  private constructor(public socket: net.Socket, private output: OutputChannel) { }


  public static async connect(address: string, logger: OutputChannel): Promise<Broadcast> {
    return new Promise((resolve, reject) => {
      const socket = net.createConnection(address, () => {
        const broadcast = new Broadcast(socket, logger);
        socket.on("data", async buffer => {
          for await (const message of pipe(
            toAsync(buffer),
            split(a => a === 10),
            map(m => Buffer.from(m)),
            filter(m => m.length > 1),
            map(m => JSON.parse(m.toString()) as Message),
          )) await broadcast.handleMessage(message);
        });
        resolve(broadcast);
      });
      socket.on("error", err => {
        reject(Error(`Failed to connect to XBase Broadcast: ${err}`));
      });
    });
  }

  private async execute(task: Task) {
    switch (task.task) {
      case "OpenLogger":
        this.output.show();
        break;
      case "ReloadLspServer":
        // TODO: Implement
        break;
      case "UpdateStatusline":
        // TODO: Implement
        break;
    }
  }

  private async notify(msg: string, level: MessageLevel) {
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
      case "Success":
        await window.showInformationMessage(msg);
        break;
    }
  }

  private async handleMessage(message: Message) {
    switch (message.type) {
      case "Notify": {
        const { msg, level } = message.args;
        await this.notify(msg, level);
        break;
      }
      case "Execute": {
        await this.execute(message.args);
        break;
      }
      case "Log": {
        const { msg, level } = message.args;
        if (level !== MessageLevel.Debug && level !== MessageLevel.Trace)
          this.output.append(msg, level);
        break;
      }
    }
  }

  dispose() {
    this.socket.pause();
    this.socket.end();
    this.socket.destroy();
  }
}
