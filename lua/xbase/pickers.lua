local M = {}
local a = require "telescope.actions"
local action_set = require "telescope.actions.set"
local s = require "telescope.actions.state"
local finder = require("telescope.finders").new_table
local picker = require("telescope.pickers").new
local sorter = require("telescope.config").values.generic_sorter
local maker = require("telescope.pickers.entry_display").create
local xbase = require "xbase"
local watch = require "xbase.watch"
local themes = require "telescope.themes"

local mappings = function(_, _)
  action_set.select:replace(function(bufnr, direction)
    a.close(bufnr)
    local entry = s.get_selected_entry()
    entry.direction = direction

    if entry.command == "Build" then
      xbase.build(entry)
    elseif entry.command == "Watch" then
      xbase.watch(entry)
    elseif entry.command == "Run" then
      xbase.run(entry)
    end
  end)

  return true
end

local insert_entry = function(acc, picker, command, target, configuration, device)
  local item = {
    command = command,
    config = { target = target, configuration = configuration },
  }

  if command == "Run" then
    item.device = device
  end

  if picker == "Watch" then
    item.command = "Watch"

    if watch.is_watching(item.config, command) then
      item.ops = "Stop"
      item.kind = command
    else
      item.ops = "Start"
      item.kind = command
    end
  end

  acc[#acc + 1] = item
end

local get_selections = function(picker)
  local commands = picker == "Watch" and { "Build", "Run" } or { picker }
  local root = vim.loop.cwd()
  local info = vim.g.xbase.projects[root]
  if info == nil then
    error "No info available"
  end

  local targets = {}

  -- TOOD(core): Support custom schemes
  for name, _ in pairs(info.targets) do
    targets[#targets + 1] = name
  end

  -- TOOD(core): Support custom project configurations
  local configurations = { "Debug", "Release" }

  local devices = {}

  if picker == "Run" or picker == "Watch" then
    -- TODO(nvim): Only include devices that is actually supported by target
    devices = vim.tbl_map(function(device)
      return {
        name = device.info.name,
        udid = device.info.udid,
      }
    end, vim.g.xbase.devices)
  end

  local results = {}

  for _, command in ipairs(commands) do
    for _, target in ipairs(targets) do
      for _, configuration in ipairs(configurations) do
        if #devices ~= 0 and command == "Run" then
          for _, device in ipairs(devices) do
            insert_entry(results, picker, command, target, configuration, device)
          end
        else
          insert_entry(results, picker, command, target, configuration)
        end
      end
    end
  end

  return results
end

local entry_maker = function(entry)
  local config = ("(%s)"):format(entry.config.configuration)
  local target = entry.config.target
  local device
  entry.ordinal = ""

  if entry.device then
    device = entry.device.name
  end

  local items, parts = {}, {}
  local ti = table.insert

  if entry.ops then
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

  ti(items, { width = 9 })
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
        a.close(bufnr)
        local selected = s.get_selected_entry()
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
