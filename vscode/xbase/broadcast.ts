import net from "net";
import { Disposable, window } from "vscode";
import type { Message, MessageLevel, Task } from "./types";
import OutputChannel from "./ui/outputChannel";

export default class Broadcast implements Disposable {

  private constructor(public socket: net.Socket, private output: OutputChannel) { }


  public static async connect(address: string, logger: OutputChannel): Promise<Broadcast> {
    return new Promise((resolve, reject) => {
      const socket = net.createConnection(address, () => {
        const broadcast = new Broadcast(socket, logger);
        socket.on("data", buffer => {
          const message = JSON.parse(`${buffer}`) as Message;
          broadcast.handleMessage(message);
        });
        resolve(broadcast);
      });
      socket.on("error", err => {
        reject(Error(`Failed to connect to XBase Broadcast: ${err}`));
      });
    });
  }

  private execute(task: Task) {
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

  private notify(msg: string, level: MessageLevel) {
    switch (level) {
      case "Info":
        window.showInformationMessage(msg);
        break;
      case "Warn":
        window.showWarningMessage(msg);
        break;
      case "Error":
        window.showErrorMessage(msg);
        break;
      case "Success":
        window.showInformationMessage(msg);
        break;
    }
  }

  private handleMessage(message: Message) {
    switch (message.type) {
      case "Notify": {
        const { msg, level } = message.args;
        this.notify(msg, level);
        break;
      }
      case "Execute": {
        this.execute(message.args);
        break;
      }
      case "Log": {
        const { msg, level } = message.args;
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
