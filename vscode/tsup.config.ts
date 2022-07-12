import { defineConfig } from "tsup";

export default defineConfig({
  entry: [
    "xbase/init.ts",
  ],
  format: ["cjs"],
  shims: false,
  dts: false,
  external: [
    "vscode",
  ],
});
