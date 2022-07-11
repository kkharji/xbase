import { Ok, Err } from "@sniptt/monads";
import XBaseServer from "./server";
import { Result, Runners } from "./types";

export default class XBaseState {
  public runners: Runners;

  private constructor(runners: Runners) {
    this.runners = runners;
  }

  static async init(server: XBaseServer): Promise<Result<XBaseState>> {
    return server.request({ method: "get_runners" })
      .then((response) => {
        return response.andThen(v => {
          if (v.isNone())
            return Err(Error("No Runners received"));
          else
            return Ok(v.unwrap());
        }).map(v => {
          return new XBaseState(v as Runners);
        });
      });
  }
}
