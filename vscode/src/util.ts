import { BuildSettings, DeviceLookup } from "./types";

/*
  * Get a capitalized named of last part of the path
*/
export function projectName(path: string): string {
  const pathArray = path.split("/");
  const lastIndex = pathArray.length - 1;
  const name = pathArray[lastIndex];
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
