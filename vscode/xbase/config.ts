import { workspace } from "vscode";

// Main XBase Configuration object
export default {
  get lsp() {
    return {
      path: getConfig("sourcekit-lsp", "path", ""),
      arguments: getConfig("sourcekit-lsp", "arguments", [""]),
    };
  },
  get ui() {
    return {
      openLoggerOnError: getConfig("ui", "openLoggerOnError", false),
    };
  }
};

const getConfig = <T>(group: string, key: string, defaultValue: T) =>
  workspace.getConfiguration(`xbase.${group}`).get(key, defaultValue);
