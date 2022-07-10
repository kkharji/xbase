import { defineConfig } from "tsup";

export default defineConfig({
  entry: [
    "src/*.ts",
  ],
  format: ["cjs"],
  shims: false,
  dts: false,
  external: [
    "vscode",
  ],
});
