local M = {}
local server = require "xbase.server"
local state = require "xbase.state"

---@alias XBaseCommand
---| '"Run"'
---| '"Build"'
---| '"Watch"'
M.Command = {
  Run = "Run",
  Build = "Build",
  Watch = "Watch",
}

local Run, Build, Watch = M.Command.Run, M.Command.Build, M.Command.Watch

---Supported Actions
---@type { value: XBaseCommand }[]
M.action_entries = (function()
  local l = {}
  for key, _ in pairs(M.Command) do
    table.insert(l, { value = key })
  end

  return l
end)()

---@class XBaseSelectEntry
---@field root string project root
---@field command XBaseCommand command to run root
---@field kind XBaseCommand command to run root
---@field settings XBaseSettings project build settings
---@field device XBaseDeviceLookup | nil device to run with
---@field operation string operation to run "Watch" | "Stop" | "Once"

---Run command in server
---@param entry XBaseSelectEntry
M.run_command = function(entry)
  server.request {
    method = string.lower(entry.command),
    args = {
      root = entry.root,
      settings = entry.settings,
      operation = entry.operation,
      device = entry.device,
    },
  }
end

---Insert New Entry
---@param root string the root of the project
---@param picker XBaseCommand the name of the picker
---@param command XBaseCommand the command to run
---@param target string the name of the target
---@param configuration string the name configuration
---@param device XBaseDeviceLookup? the device to include
---@return XBaseSelectEntry
local create_entry = function(root, picker, command, target, configuration, device)
  local item = {
    root = root,
    command = command,
    settings = { target = target, configuration = configuration },
    operation = "Once",
    device = command == M.Command.Run and device or nil,
  }

  if picker == M.Command.Watch then
    if state.is_watching(item.settings, command, item.device, root) then
      item.operation = "Stop"
      item.kind = command
    else
      item.operation = "Watch"
      item.kind = command
    end
  end

  return item
end

---Iterate over commands and targets
---@param commands XBaseCommand[]
---@param targets table
---@param include_devices boolean whether to include devices
---@param exec fun(command: string, target: string, configuration: string, device: XBaseDeviceLookup|nil)
local iterate = function(commands, targets, include_devices, exec)
  for _, command in ipairs(commands) do
    for target, info in pairs(targets) do
      local devices = (command == Run and include_devices) and state.devices(info.platform) or nil
      for _, configuration in ipairs(info.configurations) do
        exec(command, target, configuration, devices)
      end
    end
  end
end

---Generate picker selections
---@param root string the project root
---@param picker_name XBaseCommand
---@return XBaseSelectEntry[]
M.generate_entries = function(root, picker_name)
  local results = {}
  local project_info = require("xbase.state").project_info[root]
  local commands = picker_name == Watch and { Build, Run } or { picker_name }
  local include_devices = picker_name == Run or picker_name == Watch

  if project_info.targets == nil then
    error "No targets found"
  end

  iterate(commands, project_info.targets, include_devices, function(command, target, configuration, devices)
    if devices then
      local entries = vim.tbl_map(function(device)
        return create_entry(root, picker_name, command, target, configuration, device)
      end, devices)

      vim.list_extend(results, entries)
    else
      table.insert(results, create_entry(root, picker_name, command, target, configuration))
    end
  end)

  -- TODO: Keep prioritize last used device
  return results
end

---Prepare entry for selection
---@param entry XBaseSelectEntry
---@return table
---@return table
---@return table
M.prep_entry = function(entry)
  local config = ("(%s)"):format(entry.settings.configuration)
  local target = entry.settings.target
  local device = entry.device and entry.device.name or nil

  local items, parts = {}, {}
  local insert = table.insert

  entry.ordinal = ""

  if entry.operation and entry.operation ~= "Once" then
    entry.ordinal = string.format("%s %s", entry.ordinal, entry.operation)
    local operation = string.format("%s", entry.operation)

    insert(items, { width = 7 })
    insert(parts, { operation, "TSNone" })
  end

  if entry.kind then
    entry.ordinal = string.format("%s %s", entry.ordinal, entry.kind)
    insert(items, { width = 7 })
    insert(parts, { entry.kind, "TSNone" })
  end

  insert(items, { width = 20 })
  entry.ordinal = string.format("%s %s", entry.ordinal, target)
  insert(parts, { target, "TSCharacter" })

  if device then
    entry.ordinal = string.format("%s %s", entry.ordinal, device)
    insert(items, { width = 30 })
    insert(parts, { device, "TelescopeResultsClass" })
  end

  entry.ordinal = string.format("%s %s", entry.ordinal, config)
  insert(items, { width = 9 })
  insert(parts, { config, "TelescopeResultsIdentifier" })

  return entry, items, parts
end

return M
