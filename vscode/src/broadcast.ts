import net from "net";
import { Err, Ok } from "@sniptt/monads/build";
import { window } from "vscode";
import type { Message, MessageLevel, Result, Task } from "./types";
import XBaseOutputChannel from "./ui/output";

export default class XBaseBroadcast {

  private constructor(public socket: net.Socket, private output: XBaseOutputChannel) { }

  public static async connect(address: string, logger: XBaseOutputChannel): Promise<Result<XBaseBroadcast>> {
    return new Promise((resolve) => {
      const socket = net.createConnection(address);
      socket.on("error", (err) => {
        resolve(Err(Error(`Failed to connect to XBase Broadcast: ${err}`)));
      });
      socket.on("connect", () => {
        const broadcast = new XBaseBroadcast(socket, logger);
        socket.on("data", (buffer) => {
          const message = JSON.parse(`${buffer}`) as Message;
          broadcast.handleMessage(message);
        });
        resolve(Ok(broadcast));
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
}
