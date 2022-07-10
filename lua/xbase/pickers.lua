local M = {}
local a = require "telescope.actions"
local action_set = require "telescope.actions.set"
local s = require "telescope.actions.state"
local finder = require("telescope.finders").new_table
local picker = require("telescope.pickers").new
local sorter = require("telescope.config").values.generic_sorter
local maker = require("telescope.pickers.entry_display").create
local server = require "xbase.server"
local themes = require "telescope.themes"
local util = require "xbase.util"
local state = require "xbase.state"

local C = {
  Run = "Run",
  Build = "Build",
  Watch = "Watch",
}

local mappings = function(_, _)
  action_set.select:replace(function(bufnr, _)
    a.close(bufnr)
    local entry = s.get_selected_entry()
    server.request {
      method = string.lower(entry.command),
      args = {
        root = entry.root,
        settings = entry.settings,
        operation = entry.operation,
        device = entry.device,
      },
    }
  end)

  return true
end

local insert_entry = function(acc, picker, command, target, configuration, watchlist, device)
  local item = {
    command = command,
    settings = { target = target, configuration = configuration },
  }

  if command == C.Run then
    item.device = device
  end

  if picker == C.Watch then
    if util.is_watching(item.settings, command, item.device, watchlist) then
      item.operation = "Stop"
      item.kind = command
    else
      item.operation = "Watch"
      item.kind = command
    end
  else
    item.operation = "Once"
  end
  if not item.root then
    item.root = vim.loop.cwd()
  end

  acc[#acc + 1] = item
end

local get_selections = function(project_info, picker)
  local commands = picker == C.Watch and { C.Build, C.Run } or { picker }
  local targets = project_info.targets
  local include_devices = picker == C.Run or picker == C.Watch
  local watchlist = picker == "Watch" and project_info.watchlist or {}

  if targets == nil then
    error "No targets found"
  end

  -- TOOD(core): Support custom project configurations and schemes
  local configurations

  if vim.loop.fs_stat(vim.loop.cwd() .. "/Package.swift") then
    configurations = { "Debug" }
  else
    configurations = { "Debug", "Release" }
  end

  local results = {}

  for _, command in ipairs(commands) do
    for target, target_info in pairs(targets) do
      for _, configuration in ipairs(configurations) do
        local devices = state.runners[target_info.platform]
        if include_devices and command == C.Run and not (devices == nil or #devices == 0) then
          for _, device in ipairs(devices) do
            insert_entry(results, picker, command, target, configuration, watchlist, device)
          end
        else
          insert_entry(results, picker, command, target, configuration, watchlist)
        end
      end
    end
  end

  -- TODO: Keep prioritize last used device
  return results
end

local entry_maker = function(entry)
  local config = ("(%s)"):format(entry.settings.configuration)
  local target = entry.settings.target
  local device
  entry.ordinal = ""

  if entry.device then
    device = entry.device.name
  end

  local items, parts = {}, {}
  local ti = table.insert

  if entry.operation and entry.operation ~= "Once" then
    entry.ordinal = string.format("%s %s", entry.ordinal, entry.operation)
    local operation = string.format("%s", entry.operation)

    ti(items, { width = 7 })
    ti(parts, { operation, "TSNone" })
  end

  if entry.kind then
    entry.ordinal = string.format("%s %s", entry.ordinal, entry.kind)
    ti(items, { width = 7 })
    ti(parts, { entry.kind, "TSNone" })
  end

  ti(items, { width = 20 })
  entry.ordinal = string.format("%s %s", entry.ordinal, target)
  ti(parts, { target, "TSCharacter" })

  if device then
    entry.ordinal = string.format("%s %s", entry.ordinal, device)
    ti(items, { width = 30 })
    ti(parts, { device, "TelescopeResultsClass" })
  end

  entry.ordinal = string.format("%s %s", entry.ordinal, config)
  ti(items, { width = 9 })
  ti(parts, { config, "TelescopeResultsIdentifier" })

  entry.display = function(_)
    return maker {
      separator = " ",
      hl_chars = { ["|"] = "TelescopeResultsNumber" },
      items = items,
    }(parts)
  end

  return entry
end

local find = function(name, opts)
  opts = themes.get_dropdown(opts or {})
  opts.root = opts.root or vim.loop.cwd()
  local _find = function(opts)
    picker(opts, {
      prompt_title = opts.name,
      sorter = sorter {},
      finder = finder {
        results = get_selections(opts.project_info, name),
        entry_maker = entry_maker,
      },
      attach_mappings = mappings,
    }):find()
  end

  if opts.project_info then
    _find(opts)
  else
    server.get_project_info(opts.root, function(project_info)
      opts.project_info = project_info
      _find(opts)
    end)
  end
end

M.watch = function(opts)
  find(C.Watch, opts)
end

M.build = function(opts)
  find(C.Build, opts)
end

M.run = function(opts)
  find(C.Run, opts)
end

M.actions = function(opts)
  opts = require("telescope.themes").get_dropdown(opts or {})
  opts.root = opts.root or vim.loop.cwd()
  server.get_project_info(opts.root, function(project_info)
    opts.project_info = project_info

    picker(opts, {
      sorter = sorter {},
      prompt_title = "Pick Xbase Action Category",
      finder = finder {
        results = {
          { value = C.Build },
          { value = C.Watch },
          { value = C.Run },
        },
        entry_maker = function(entry)
          entry.ordinal = entry.value
          entry.display = function(e)
            local opts = {}

            opts.separator = " "
            opts.hl_chars = { ["|"] = "TelescopeResultsNumber" }
            opts.items = { { width = 40 } }

            return maker(opts) { { e.value, "TelescopeResultsMethod" } }
          end

          return entry
        end,
      },
      attach_mappings = function(_, _)
        a.select_default:replace(function(bufnr)
          local selected = s.get_selected_entry()

          a.close(bufnr)
          if not selected then
            print "No selection"
            return
          end

          if selected.value == C.Watch then
            M.watch(opts)
          elseif selected.value == C.Build then
            M.build(opts)
          elseif selected.value == C.Run then
            M.run(opts)
          end
        end)
        return true
      end,
    }):find()
  end)
end

return M
