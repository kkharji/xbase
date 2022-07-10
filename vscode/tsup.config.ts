import { defineConfig } from "tsup";

export default defineConfig({
  entry: [
    "src/index.ts",
    "src/types.ts",
    "src/server.ts",
  ],
  format: ["cjs"],
  shims: false,
  dts: false,
  external: [
    "vscode",
  ],
});
