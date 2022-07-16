import { Disposable, StatusBarAlignment, StatusBarItem, ThemeColor, window } from "vscode";
import { ContentLevel } from "../types";

export interface SetStatuslineArg {
  content: string;
  level?: ContentLevel;
  is_success?: boolean;
  icon?: string
}

/**
 * Display xbase status
 */
export default class Statusline implements Disposable {
  private item: StatusBarItem;
  private infoColor = new ThemeColor("foreground");
  private errorColor = new ThemeColor("editorError.foreground");
  private warnColor = new ThemeColor("editorWarning.foreground");
  private extName = "XBase";

  constructor() {
    this.item = window.createStatusBarItem(StatusBarAlignment.Left);
  }

  public update({ content, level = "Info", icon = "$(sync~spin)" }: SetStatuslineArg) {
    this.set({ icon, content, level });
  }

  public setDefault() {
    this.set({ content: "" });
  }

  public set({ content, level = "Info", is_success, icon = "$(eye)" }: SetStatuslineArg) {
    this.item.text = `${icon} ${this.extName} ${content}`;
    this.item.show();
    this.setColor(level, is_success);
  }

  private setColor(level: ContentLevel, is_success?: boolean) {
    if (is_success !== undefined && is_success === false) {
      this.item.color = this.errorColor;
    } else {
      switch (level) {
        case "Error":
          this.item.color = this.errorColor;
          break;
        case "Warn":
          this.item.color = this.warnColor;
          break;
        case "Info":
        case "Debug":
        case "Trace":
          this.item.color = this.infoColor;
          break;

      }
    }
  }

  dispose() {
    this.dispose();
  }

}
