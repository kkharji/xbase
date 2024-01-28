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
  },

  get devices(): { iOS: string[], watchOS: string[], tvOS: string[], xrOS: string[] } {
    return getConfig("simctl", "devices", {
      iOS: getConfig("simctl", "iOS", []),
      watchOS: getConfig("simctl", "watchOS", []),
      tvOS: getConfig("simctl", "tvOS", []),
      xrOS: getConfig("simctl", "xrOS", [])
    });
  }
};

const getConfig = <T>(group: string, key: string, defaultValue: T) =>
  workspace.getConfiguration(`xbase.${group}`).get(key, defaultValue);
