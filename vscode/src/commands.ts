/* eslint-disable @typescript-eslint/no-unused-vars */
import { Ok, Err, } from "@sniptt/monads";
import { commands, window, workspace } from "vscode";
import XBaseServer from "./server";
import XBaseState from "./state";
import { BuildSettings, DeviceLookup, Operation, ProjectInfo, Request, Result } from "./types";
import * as util from "./util";

interface PickerItem {
  label: string,
  method: string,
  detail: string,
  description: string,
  settings: BuildSettings,
  device?: DeviceLookup,
  operation: Operation,
};

export default class XBaseCommands {
  constructor(private server: XBaseServer, private state: XBaseState) {
    commands.registerCommand("xbase.build", () => this.serverExecute("Build"));
    commands.registerCommand("xbase.run", () => this.serverExecute("Run"));
    commands.registerCommand("xbase.watch", () => this.serverExecute("Watch"));
  }

  /**
    * Get Project Information for a given `root`
  */
  private async getProjectInfo(root: string): Promise<ProjectInfo | undefined> {
    return this.server.request({ method: "get_project_info", args: { root } })
      .then(response => response.andThen(inner => inner.match({
        some: value => Ok(value as ProjectInfo),
        none: () => Err(Error(`No Project info for ${root}`)) as Result<ProjectInfo>
      })))
      .then(result => {
        if (result.isOk()) return result.unwrap();
        const msg = `Failed to get project inf: ${result.unwrapErr()}`;
        console.error(msg);
        window.showErrorMessage(msg);
      });
  }

  /**
    * Execute xbase command
    * TODO: Refactor to get available configurations with projectInfo from server
  */
  public async serverExecute(command: string) {
    const title = `XBase ${command}`;
    const root = workspace.workspaceFolders![0].uri.fsPath;
    const isWatchCommand = (command === "Watch");
    const commands = isWatchCommand ? ["Build", "Run"] : [command];
    const projectInfo = await this.getProjectInfo(root);

    if (projectInfo === undefined) return;

    const pickerItems: PickerItem[] = [];
    const cfgList = ["Debug", "Release"];

    forEach(commands, cfgList, Object.keys(projectInfo.targets))
      ((command, configuration, target) => {
        const runners = (command === "Run") ? this.state.runners[projectInfo.targets[target].platform] : undefined;
        const baseDetail = `${command} ${target} with ${configuration}`;
        const baseItem: PickerItem = {
          label: target,
          description: `(${configuration})`,
          detail: "",
          method: command.toLowerCase(),
          settings: { target, configuration, scheme: null },
          operation: Operation.Once,
        };

        const isWatching = (device?: DeviceLookup) => {
          if (isWatchCommand)
            return util.isWatching(root, command, baseItem.settings, projectInfo.watchlist, device);
        };

        const getDetail = (isWatching?: boolean, device?: DeviceLookup) => {
          let detail = `${baseDetail}`;
          if (device) detail += `on ${device.name}`;
          if (isWatching !== undefined) {
            const watchAction = isWatching ? "Stop" : "Watch";
            detail = `${watchAction} ${detail}`;
          }
          return detail;
        };

        if (runners) {
          runners.forEach(device => {
            const isWatching_ = isWatching(device);
            if (isWatching_ !== undefined)
              baseItem.operation = isWatching_ ? Operation.Stop : Operation.Watch;
            return pickerItems.push({
              ...baseItem,
              device: device,
              label: `${target} on ${device.name}`,
              detail: getDetail(isWatching_, device),
              operation: isWatching_ ? Operation.Stop : Operation.Watch
            });
          });
        }

        const isWatching_ = isWatching();
        if (isWatching_ !== undefined)
          baseItem.operation = isWatching_ ? Operation.Stop : Operation.Watch;
        baseItem.detail = getDetail(isWatching_);
        return pickerItems.push(baseItem);

      });

    const entry = await window.showQuickPick(pickerItems, { title, matchOnDescription: true });
    console.debug(entry);

    if (entry) {
      const { settings, operation, device } = entry;
      await this.server.request({
        method: entry.method,
        args: { root, settings, operation, device, }
      } as Request);
    };

  }
}

const forEach = (commands: string[], cfgList: string[], targets: string[]) =>
  (fn: (command: string, configuration: string, target: string) => void) =>
    commands.forEach(command =>
      cfgList.forEach(configuration =>
        targets.forEach(target =>
          fn(command, configuration, target))));
