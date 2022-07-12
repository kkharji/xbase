/* eslint-disable @typescript-eslint/no-unused-vars */
import { commands, window } from "vscode";
import type { BuildSettings, DeviceLookup, ProjectInfo, Request } from "./types";
import { Operation } from "./types";
import * as util from "./util";
import { WorkspaceContext } from "./workspaceContext";

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
  projectInfo: ProjectInfo,
  ctx: WorkspaceContext,
): PickerItem[] {
  const pickerItems: PickerItem[] = [];
  const isWatchCommand = (command === "Watch");
  const cmds = isWatchCommand ? ["Build", "Run"] : [command];
  const { targets, watchlist } = projectInfo;
  // TODO: make cfgList part of targets
  const cfgList = ["Debug", "Release"];

  const forEach = (fn: (cmd: string, cfg: string, target: string) => void
  ) => cmds.forEach(
    cmd => cfgList.forEach(
      cfg => Object.keys(targets).forEach(
        target => fn(cmd, cfg, target))));

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
      ? (isWatching ? Operation.Stop : Operation.Watch)
      : Operation.Once;

    const getDetail = (isWatching?: boolean, device?: DeviceLookup) => {
      let detail = `${baseDetail}`;
      if (device)
        detail += `on ${device.name}`;
      if (isWatching !== undefined) {
        const watchAction = isWatching ? "Stop" : "Watch";
        detail = `${watchAction} ${detail}`;
      }
      return detail;
    };

    if (runners) {
      runners.forEach(device => {
        const item = { ...baseItem };
        const isWatching = isWatchingProject(device);
        item.device = device;
        item.label = `${target} on ${device.name}`;
        item.detail = getDetail(isWatching, device);
        item.operation = getOperation(isWatching);
        return pickerItems.push(item);
      });
    }

    const isWatching = isWatchingProject();
    baseItem.detail = getDetail(isWatching);
    baseItem.operation = getOperation(isWatching);

    return pickerItems.push(baseItem);

  });
  return pickerItems;
}

async function serverExecute(command: string, ctx: WorkspaceContext) {
  try {
    const root = ctx.currentFolder?.uri.fsPath;
    if (root === undefined) {
      window.showErrorMessage(`${command}: No project root found`);
      return;
    }
    const projectInfo = await ctx.server.getProjectInfo(root);
    const entry = await window.showQuickPick(
      getPickerItems(root, command, projectInfo, ctx),
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
  } catch (error) {
    window.showErrorMessage(`${command} Failed: ${error}`);
  }

}


export function register(ctx: WorkspaceContext) {
  ctx.subscriptions.push(
    commands.registerCommand("xbase.build", () => serverExecute("Build", ctx)),
    commands.registerCommand("xbase.run", () => serverExecute("Run", ctx)),
    commands.registerCommand("xbase.watch", () => serverExecute("Watch", ctx)),
  );
}
