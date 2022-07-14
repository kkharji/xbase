import type vscode from "vscode";
import { window } from "vscode";
import { ContentLevel } from "../types";

export default class OutputChannel implements vscode.Disposable {
  private channel: vscode.OutputChannel;
  private shown = false;

  constructor() {
    this.channel = window.createOutputChannel("XBase");
  }

  dispose() {
    this.channel.dispose();
  }

  /* show output */
  public show() {
    this.channel.show(true);
    this.channel.hide();
  }
  public toggle() {
    if (this.shown) {
      this.channel.hide();
      this.shown = false;
    } else {
      this.channel.show(true);
      this.shown = true;
    }
  }

  // TODO: output source code warnings & errors to Problems
  append(msg: string, level: ContentLevel) {
    const line = `${this.timestamp}: ${msg}`;

    // TODO: find out based on vscode current log level
    this.channel.appendLine(line);
    switch (level) {
      case "Error": console.error(line); break;
      case "Warn": console.warn(line); break;
      case "Debug": console.debug(line); break;
      case "Info": console.info(line); break;
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
