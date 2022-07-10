import net from "net";
import type { Option } from "@sniptt/monads/build";
import { Err, None, Ok, Some } from "@sniptt/monads/build";
import type { Request, Response, Result } from "./types";
import XBaseBroadcast from "./broadcast";

export default class XBaseServer {
  roots: string[] = [];
  broadcasts: XBaseBroadcast[] = [];

  private constructor(public socket: net.Socket) { }

  public static async connect(): Promise<Result<XBaseServer>> {
    return new Promise((resolve) => {
      const socket = net.createConnection("/tmp/xbase.socket");
      socket.on("error", (err) => {
        // TODO: Spawn xbase socket
        resolve(Err(Error(`Failed to connect to xbase socket: ${err}`)));
      });
      socket.on("connect", () => resolve(Ok(new XBaseServer(socket))));
    });
  }

  // Send a new request
  private request(req: Request): Promise<Result<Option<unknown>>> {
    const { socket } = this;
    const data = JSON.stringify(req);

    return new Promise((resolve) => {
      socket.write(`${data}\n`, (error) => {
        if (error !== undefined) {
          return resolve(Err(error));
        }
        else {
          console.debug(`Sent ${data}`);
          socket.once("data", (buffer) => {
            const { error, data } = JSON.parse(`${buffer}`) as Response;
            if (error !== null && error !== undefined) {
              const { kind, msg } = error;
              resolve(Err(Error(`${kind}: ${msg}`)));
            }
            else if (data !== null) {
              resolve(Ok(Some(data)));
            }
            else {
              resolve(Ok(None));
            }
          });
        }
      });
    });
  }

  // Register a given root
  async register(root: string): Promise<Result<null>> {
    const response = await this.request({ method: "register", args: { root } });
    const broadcast_address = response.andThen((v) => {
      if (v.isNone())
        {return Err(Error("Registeration request returned none!"));}
      else
        {return Ok(v.unwrap());}
    }).map(v => v as string);

    if (broadcast_address.isErr())
      {return Err(broadcast_address.unwrapErr());}

    const broadcast_connect = await XBaseBroadcast.connect(broadcast_address.unwrap());

    if (broadcast_connect.isErr()) {
      const error = broadcast_connect.unwrapErr();
      return Err(Error(`Failed to connect to broadcast server ${error}`));
    }

    this.broadcasts.push(broadcast_connect.unwrap());

    return Ok(null);
  }
}
