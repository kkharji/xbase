import vscode, { DiagnosticSeverity, Position, Range, Uri } from "vscode";
import { ContentLevel } from "../types";



export default class Logger implements vscode.Disposable {
  private channel = vscode.window.createOutputChannel("XBase", "xclog");
  public diagnosticCollection = vscode.languages.createDiagnosticCollection("XBase");
  private problemRegex = /\[(.*)\] (.*):(\d+):(\d+):(?:\s(.*):)?\s+(.*)$/;

  // TODO: find a more accurate way to check whether output window is shown
  private shown = false;

  /* show output */
  public async toggle() {
    await vscode.commands.executeCommand("workbench.action.output.toggleOutput");
    await vscode.commands.executeCommand("workbench.action.focusFirstEditorGroup");
    if (this.shown) {
      this.shown = false;
      this.channel.hide();
    } else {
      this.channel.show(true);
      this.shown = true;
    }
  }

  append(line: string, level: ContentLevel = ContentLevel.Debug) {
    const groups = this.problemRegex.exec(line);

    if (groups) {
      const filePath = Uri.file(groups[2]);
      const lineNr = parseInt(groups[3]);
      const colNr = parseInt(groups[4]);
      const type = groups[5] ? groups[5] : groups[1].toLowerCase();
      const message = groups[6];
      const currentDiagnostics = this.diagnosticCollection.get(filePath);
      const diagnostics = [...(currentDiagnostics ? currentDiagnostics : [])];

      let severity;
      if (type === "error")
        severity = DiagnosticSeverity.Error;
      else if (type === "warn" || type === "warning")
        severity = DiagnosticSeverity.Warning;
      else
        severity = DiagnosticSeverity.Information;

      // TODO: Ensure that this message doesn't already exists in problems!
      diagnostics.push({
        message,
        range: new Range(
          new Position(lineNr, colNr),
          new Position(lineNr, colNr + 10)
        ),
        severity,
      });

      this.diagnosticCollection.set(filePath, diagnostics);

    } else if (/\[(Error|Warning)\]/.test(line) === false) {
      this.channel.appendLine(line);
      switch (level) {
        case "Error": console.error(line); break;
        case "Warn": console.warn(line); break;
        case "Debug": console.debug(line); break;
        case "Info": console.info(line); break;
      }
    }
  }

  dispose() {
    this.channel.dispose();
    this.diagnosticCollection.dispose();
  }
}
