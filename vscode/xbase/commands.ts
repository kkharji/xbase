/* eslint-disable @typescript-eslint/no-unused-vars */
import { commands, window } from "vscode";
import type { BuildSettings, DeviceLookup, Request } from "./types";
import { Operation } from "./types";
import * as util from "./util";
import { WorkspaceContext } from "./workspaceContext";
import configurations from "./config";

interface PickerItem {
  label: string,
  method: string,
  detail?: string,
  description: string,
  settings: BuildSettings,
  device?: DeviceLookup,
  operation: Operation,
};

function getPickerItems(
  root: string,
  command: string,
  ctx: WorkspaceContext,
): PickerItem[] {
  const pickerItems: PickerItem[] = [];
  const isWatchCommand = (command === "Watch");
  const cmds = isWatchCommand ? ["Build", "Run"] : [command];
  const folderCtx = ctx.folders.find(f => f.uri.fsPath === root);
  if (folderCtx === undefined) {
    window.showErrorMessage("No folder context found, please report issue");
    return [];
  }
  const { targets, watchlist } = folderCtx.projectInfo;
  const devices = configurations.devices;

  const forEach = (fn: (cmd: string, cfg: string, target: string) => void) =>
    cmds.forEach(cmd =>
      Object.keys(targets).forEach(target =>
        targets[target].configurations.forEach(cfg =>
          fn(cmd, cfg, target))));

  forEach((command, configuration, target) => {
    const platform = targets[target].platform;
    const runners = (command === "Run") ? ctx.runners[platform] : undefined;
    const baseDetail = `${command} ${target} with ${configuration}`;
    const description = `(${configuration})`;
    const settings = { target, configuration, scheme: null };
    const label = target;
    const method = command.toLowerCase();
    const baseItem = { label, description, method, settings, } as PickerItem;

    const isWatchingProject = (device?: DeviceLookup) => isWatchCommand
      ? util.isWatching(root, command, settings, watchlist, device)
      : undefined;

    const getOperation = (isWatching?: boolean) => (isWatching !== undefined)
      ? (isWatching ? "Stop" : "Watch") : "Once";

    const getDetail = (isWatching?: boolean, device?: DeviceLookup) => {
      let detail = `${baseDetail}`;
      if (device)
        detail += ` on ${device.name}`;
      if (isWatching !== undefined) {
        const watchAction = isWatching ? "Stop" : "Watch";
        detail = `${watchAction} ${detail}`;
      }
      return detail;
    };

    if (runners) {
      let platformRunners: DeviceLookup[] = [];
      let platformDevices: string[];

      switch (platform) {
        case "iOS":
          platformDevices = devices.iOS;
          break;
        case "watchOS":
          platformDevices = devices.watchOS;
          break;
        default:
          platformDevices = devices.tvOS;
          break;
      }

      if (platformDevices.length !== 0) {
        platformRunners = runners.filter(device => {
          return platformDevices.includes(device.name);
        });
        if (platformRunners.length == 0) {
          const configuredDevicesNames = platformDevices.join(", ");
          const avaliableDeviceNames = runners.map(v => v.name).join(", ");
          console.error(`No runners available based on user config. config: ${configuredDevicesNames}, available: ${avaliableDeviceNames}`);
        }
      } else {
        platformRunners = runners;
      }

      platformRunners.forEach(device => {
        console.log(device);
        const item = { ...baseItem };
        const isWatching = isWatchingProject(device);
        item.device = device;
        item.label = `${target} on ${device.name}`;
        item.detail = getDetail(isWatching, device);
        item.operation = getOperation(isWatching);
        pickerItems.push(item);
      });
    } else {

      const isWatching = isWatchingProject();
      baseItem.detail = getDetail(isWatching);
      baseItem.operation = getOperation(isWatching);

      return pickerItems.push(baseItem);
    }


  });
  return pickerItems;
}

async function serverExecute(command: string, ctx: WorkspaceContext) {
  ctx.logger.diagnosticCollection.clear();
  try {
    const root = ctx.currentFolder?.uri.fsPath;
    if (root === undefined) {
      window.showErrorMessage(`${command}: No project root found`);
      return;
    }
    const entry = await window.showQuickPick(
      getPickerItems(root, command, ctx),
      {
        title: `XBase ${command}`,
        matchOnDescription: true
      }
    );

    if (entry) {
      const { settings, operation, device, method } = entry;
      const args = { root, settings, operation, device };
      await ctx.server.request({ method, args, } as Request);
    };
  } catch (err) {
    const error = err as Error;
    window.showErrorMessage(`Running \`${command}\` Failed: ${error.message}`);
  }

}


export function register(ctx: WorkspaceContext) {
  ctx.subscriptions.push(
    commands.registerCommand("xbase.build", () => serverExecute("Build", ctx)),
    commands.registerCommand("xbase.run", () => serverExecute("Run", ctx)),
    commands.registerCommand("xbase.watch", () => serverExecute("Watch", ctx)),
    commands.registerCommand("xbase.toggleLogger", () => ctx.logger.toggle()),
  );
}
