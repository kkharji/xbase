import type { OutputChannel } from "vscode";
import { window } from "vscode";
import { MessageLevel } from "../types";

export default class XBaseOutputChannel {
  private channel: OutputChannel;

  constructor() {
    this.channel = window.createOutputChannel("XBase");
  }

  /* show output */
  public show() {
    this.channel.show();
  }

  // TODO: output source code warnings & errors to Problems
  append(msg: string, level: MessageLevel) {
    const line = `${this.timestamp}: ${msg}`;

    this.channel.appendLine(line);
    switch (level) {
      case "Error":
        console.error(line);
        break;
      case "Warn":
        console.warn(line);
        break;
      case "Debug":
        console.debug(line);
        break;
      case "Info":
        console.info(line);
        break;
      case "Success":
        console.info(line);
        break;
    }
  }

  private get timestamp(): string {
    const date = new Date();
    return date.toLocaleString("en-US", {
      hourCycle: "h23",
      hour: "2-digit",
      minute: "numeric",
      second: "numeric",
    });
  }
}
