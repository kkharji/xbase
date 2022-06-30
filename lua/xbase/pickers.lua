local M = {}
local a = require "telescope.actions"
local action_set = require "telescope.actions.set"
local s = require "telescope.actions.state"
local finder = require("telescope.finders").new_table
local picker = require("telescope.pickers").new
local sorter = require("telescope.config").values.generic_sorter
local maker = require("telescope.pickers.entry_display").create
local xbase = require "xbase"
local themes = require "telescope.themes"
local util = require "xbase.util"

local mappings = function(_, _)
  action_set.select:replace(function(bufnr, direction)
    a.close(bufnr)
    local entry = s.get_selected_entry()
    entry.direction = direction

    if entry.command == "Build" then
      xbase.build(entry)
    elseif entry.command == "Run" then
      xbase.run(entry)
    end
  end)

  return true
end

local insert_entry = function(acc, picker, command, target, configuration, watchlist, device)
  local item = {
    command = command,
    settings = { target = target, configuration = configuration },
  }

  if command == "Run" then
    item.device = device
  end

  if picker == "Watch" then
    if util.is_watching(item.settings, command, item.device, watchlist) then
      item.ops = "Stop"
      item.kind = command
    else
      item.ops = "Watch"
      item.kind = command
    end
  else
    item.ops = "Once"
  end
  if not item.root then
    item.root = vim.loop.cwd()
  end

  acc[#acc + 1] = item
end

local get_selections = function(picker)
  local commands = picker == "Watch" and { "Build", "Run" } or { picker }
  local targets = xbase.targets()
  local include_devices = picker == "Run" or picker == "Watch"
  local watchlist = picker == "Watch" and xbase.watching() or {}

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
        local devices = xbase.runners(target_info.platform)
        if include_devices and command == "Run" and #devices ~= 0 then
          for _, device in ipairs(devices) do
            insert_entry(results, picker, command, target, configuration, watchlist, device)
          end
        else
          insert_entry(results, picker, command, target, configuration, watchlist)
        end
      end
    end
  end

  -- if picker == "Run" or picker == "Watch" then
  --   table.sort(results, function(a, b)
  --     if a.device and b.device then
  --       return a.device.is_on and not b.device.is_on
  --     else
  --       return
  --     end
  --   end)
  -- end

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

  if entry.ops and entry.ops ~= "Once" then
    entry.ordinal = string.format("%s %s", entry.ordinal, entry.ops)
    local ops = string.format("%s", entry.ops)

    ti(items, { width = 7 })
    ti(parts, { ops, "TSNone" })
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

M.watch = function(opts)
  opts = themes.get_dropdown(opts or {})
  picker(opts, {
    sorter = sorter {},
    prompt_title = "Watch",
    finder = finder { results = get_selections "Watch", entry_maker = entry_maker },
    attach_mappings = mappings,
  }):find()
end

M.build = function(opts)
  opts = themes.get_dropdown(opts or {})
  picker(opts, {
    sorter = sorter {},
    prompt_title = "Build",
    finder = finder { results = get_selections "Build", entry_maker = entry_maker },
    attach_mappings = mappings,
  }):find()
end

M.run = function(opts)
  opts = themes.get_dropdown(opts or {})
  picker(opts, {
    sorter = sorter {},
    prompt_title = "Run",
    finder = finder { results = get_selections "Run", entry_maker = entry_maker },
    attach_mappings = mappings,
  }):find()
end

M.actions = function(opts)
  opts = require("telescope.themes").get_dropdown(opts or {})
  picker(opts, {
    sorter = sorter {},
    prompt_title = "Pick Xbase Action Category",
    finder = finder {
      results = {
        { value = "Watch" },
        { value = "Build" },
        { value = "Run" },
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

        if selected.value == "Watch" then
          M.watch(opts)
        elseif selected.value == "Build" then
          M.build(opts)
        elseif selected.value == "Run" then
          M.run(opts)
        end
      end)
      return true
    end,
  }):find()
end

return M
