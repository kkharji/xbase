import { basename, join, relative } from "path";
import { BuildSettings, DeviceLookup } from "./types";
import { access } from "fs/promises";
import glob from "tiny-glob";

/*
  * Get a capitalized named of last part of the path
*/
export function projectName(path: string): string {
  const name = basename(path);
  return name.charAt(0).toUpperCase() + name.slice(1);
}

/*
  * Check if xbase task is watched
*/
export function isWatching(root: string, command: string, settings: BuildSettings, watchlist: string[], device?: DeviceLookup): boolean {
  let key = `${root}:${command}`;

  if (command === "Run") key += (device) ? `:${device.name}` : ":Bin";

  key += `-configuration ${settings.configuration}`;
  key += ` -target ${settings.target}`;

  return watchlist.includes(key);
};

export async function pathExists(...pathComponents: string[]): Promise<boolean> {
  try {
    await access(join(...pathComponents));
    return true;
  } catch {
    return false;
  }
}

/**
 * Return whether a file is inside a folder
 * @param subfolder child file/folder
 * @param folder parent folder
 * @returns if child file is inside parent folder
 */
export function isPathInsidePath(subfolder: string, folder: string): boolean {
  const relativePath = relative(folder, subfolder);
  // return true if path doesnt start with '..'
  return relativePath[0] !== "." || relativePath[1] !== ".";
}

/**
 * Return whether a given root is support
 */
export async function isSupportedProjectRoot(root: string): Promise<boolean> {
  return (await pathExists(root, "Project.swift")
    || await pathExists(root, "project.yml")
    || await pathExists(root, "Package.swift")
    || (await glob("*.xcodeproj", { cwd: root })).length !== 0);
}
