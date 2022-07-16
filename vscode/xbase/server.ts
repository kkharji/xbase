import net from "net";
import type { JSONValue, ProjectInfo, Request, Response } from "./types";
import { Disposable } from "vscode";

export default class Server implements Disposable {
  roots: string[] = [];

  private constructor(public socket: net.Socket) { }

  public static async connect(): Promise<Server> {
    return new Promise((resolve, reject) => {
      const socket = net.createConnection("/tmp/xbase.socket");
      // TODO: Spawn xbase socket
      socket.on("error", (err) => {
        reject(Error(`Failed to connect to xbase socket: ${err}`));
      });
      socket.on("connect", () => {
        console.log("[XBase] Server Connected");
        resolve(new Server(socket));
      });
    });
  }

  // Register a given root
  async register(root: string): Promise<string> {
    const value = await this.request({ method: "register", args: { root } })
      .catch(error => {
        throw Error(`Registeration failed: ${error}`);
      });

    if (typeof value === "string") return value;

    throw Error(`Expected response to be a string, got ${value}`);
  }

  // Drop a root project
  async drop(root: string): Promise<void> {
    await this.request({ method: "drop", args: { roots: [root] } })
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

  /**
    * Get Project Information for a given `root`
  */
  public async getProjectInfo(root: string): Promise<ProjectInfo> {
    return this.request({ method: "get_project_info", args: { root } })
      .then(result => {
        if (result !== null || result !== undefined) return result as ProjectInfo;
        throw Error("Expected server to return project Information, got nothing");
      });
  }

  dispose() {
    this.socket.pause();
    this.socket.end();
    this.socket.destroy();
  }
}
