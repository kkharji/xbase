import net from "net";
import { Err, Ok } from "@sniptt/monads/build";
import { window } from "vscode";
import type { Message, Result } from "./types";

export default class XBaseBroadcast {
  private constructor(public socket: net.Socket) { }

  private handleMessage(msg: Message) {
    switch (msg.type) {
      case "Notify":
        // TODO: handle msg types
        // window.showErrorMessage()
        window.showInformationMessage(msg.args.msg);
        break;
      case "Log":
        // TODO: handle log messages
        break;
      case "Execute":
        break;
    }
  }

  public static async connect(address: string): Promise<Result<XBaseBroadcast>> {
    return new Promise((resolve) => {
      const socket = net.createConnection(address);
      socket.on("error", (err) => {
        resolve(Err(Error(`Failed to connect to XBase Broadcast: ${err}`)));
      });
      socket.on("connect", () => {
        const broadcast = new XBaseBroadcast(socket);
        socket.on("data", (buffer) => {
          const message = JSON.parse(`${buffer}`) as Message;
          broadcast.handleMessage(message);
        });
        resolve(Ok(broadcast));
      });
    });
  }
}
