import { defineConfig } from "tsup";

export default defineConfig({
  entry: [
    "src/*.ts",
    "src/ui/*.ts",
  ],
  format: ["cjs"],
  shims: false,
  dts: false,
  external: [
    "vscode",
  ],
});
