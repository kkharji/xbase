local M = {}
local a = require "telescope.actions"
local action_set = require "telescope.actions.set"
local s = require "telescope.actions.state"
local finder = require("telescope.finders").new_table
local picker = require("telescope.pickers").new
local sorter = require("telescope.config").values.generic_sorter
local maker = require("telescope.pickers.entry_display").create
local themes = require "telescope.themes"
local xbase = require "xbase.pickers.util"

local C = xbase.Command

--- Individual Pickers
do
  local attach_mappings = function(_, _)
    action_set.select:replace(function(bufnr, _)
      a.close(bufnr)
      ---@type XBaseSelectEntry
      local entry = s.get_selected_entry()
      xbase.run_command(entry)
    end)

    return true
  end

  local entry_maker = function(entry)
    local e, items, parts = xbase.prep_entry(entry)
    e.display = function(_)
      return maker {
        separator = " ",
        hl_chars = { ["|"] = "TelescopeResultsNumber" },
        items = items,
      }(parts)
    end

    return e
  end

  ---Given action name create and run picker
  ---@param name XBaseCommand|string
  ---@param opts table
  local find = function(name, opts)
    opts = themes.get_dropdown(opts or {})
    opts.root = opts.root or vim.loop.cwd()
    picker(opts, {
      prompt_title = opts.name,
      sorter = sorter {},
      finder = finder {
        results = xbase.generate_entries(opts.root, name),
        entry_maker = entry_maker,
      },
      attach_mappings = attach_mappings,
    }):find()
  end

  ---XBase watch picker
  ---@param opts table
  M.watch = function(opts)
    find(C.Watch, opts)
  end

  ---XBase build picker
  M.build = function(opts)
    find(C.Build, opts)
  end

  ---XBase run picker
  M.run = function(opts)
    find(C.Run, opts)
  end
end
-- Indvidual Pickers

--- Action Picker
do
  --- TODO:  attach opts to telescope through telescope.actions.state?
  local action = function(opts)
    return function(bufnr)
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
    end
  end

  local actions_finder = finder {
    results = xbase.action_entries,
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
  }

  ---Select from all available xbase actions
  ---@param opts table
  M.actions = function(opts)
    opts = require("telescope.themes").get_dropdown(opts or {})
    opts.root = opts.root or vim.loop.cwd()
    picker(opts, {
      sorter = sorter {},
      prompt_title = "Pick Xbase Action Category",
      finder = actions_finder,
      attach_mappings = function(_, _)
        a.select_default:replace(action(opts))
        return true
      end,
    }):find()
  end
end
-- Action Pickers

return M
