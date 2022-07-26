import net from "net";
import type { JSONValue, Request, Response } from "./types";
import { Disposable } from "vscode";
import { spawn } from "child_process";
import { XBASE_BIN_ROOT } from "./constants";

export default class Server implements Disposable {
  roots: string[] = [];

  private static onConnect = (resolve: (value: Server) => void, socket: net.Socket) =>
    () => {
      console.log("[XBase] Server Connected");
      resolve(new Server(socket));
    };
  ;
  private constructor(public socket: net.Socket) { }

  public static async connect(): Promise<Server> {
    return new Promise((resolve, reject) => {
      const socket = net.createConnection("/tmp/xbase.socket");

      socket.on("error", () => {
        console.log("[XBase] No socket running, spawning");
        spawn(`${XBASE_BIN_ROOT}/xbase`, { detached: true, });

        // TODO: Find a batter way
        // The timeout is needed to give some time for xbase to startup
        // NOTE: child.on('spawn') doesn't cut it.
        setTimeout(() => {
          const socket = net.createConnection("/tmp/xbase.socket");
          socket.on("connect", Server.onConnect(resolve, socket));
          socket.on("error", (err) => reject(Error(`Failed to connect to xbase socket: ${err}`)));
        }, 500);
      });

      socket.on("connect", Server.onConnect(resolve, socket));
    });
  }

  // Register a given root
  async register(root: string): Promise<string> {
    const value = await this.request({ method: "register", args: { root, id: process.pid } })
      .catch(error => {
        throw Error(`Registeration failed: ${error}`);
      });

    if (typeof value === "string") return value;

    throw Error(`Expected response to be a string, got ${value}`);
  }

  // Drop a root project
  async drop(root: string): Promise<void> {
    await this.request({ method: "drop", args: { id: process.pid, roots: [root] } })
      .catch(error => {
        throw Error(`Drop failed: ${error}`);
      });
  }

  /**
    * Send a new request to xbase server
  */
  public async request(req: Request): Promise<JSONValue | undefined> {
    const { socket } = this;
    const data = JSON.stringify(req);

    return new Promise((resolve, reject) => {
      socket.write(`${data}\n`, (error) => {
        if (error !== undefined) {
          return reject(new Error);
        } else {
          socket.once("data", (buffer) => {
            const { error, data } = JSON.parse(`${buffer}`) as Response;
            if (error)
              reject(new Error(`Server Errored: (${error.kind}): ${error.msg}`));
            else
              resolve(data);
          });
        }
      });
    });
  }

  dispose() {
    this.socket.pause();
    this.socket.end();
    this.socket.destroy();
  }
}
