local M = {}
local a = require "telescope.actions"
local s = require "telescope.actions.state"
local finder = require("telescope.finders").new_table
local picker = require("telescope.pickers").new
local sorter = require("telescope.config").values.generic_sorter
local maker = require("telescope.pickers.entry_display").create
local xbase = require "xbase"
local watch = require "xbase.watch"

local handle_action = function(bufnr)
  a.close(bufnr)
  local selected = s.get_selected_entry()

  if selected.command == "Build" then
    xbase.build(selected)
  elseif selected.command == "Watch" then
    xbase.watch(selected)
  end
end

local get_current_dir_targets = function()
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

  return targets, configurations
end

M.watch = function(opts)
  opts = require("telescope.themes").get_dropdown(opts or {})
  local targets, configurations = get_current_dir_targets()
  local commands = { "Build", "Run" }

  picker(opts, {
    sorter = sorter {},
    prompt_title = "Watch",
    finder = finder {
      results = (function()
        local results = {}

        --- TODO(nvim): Make nested picker based on available commands
        for _, target in ipairs(targets) do
          for _, command in ipairs(commands) do
            for _, configuration in ipairs(configurations) do
              local config = configuration == "Debug" and "" or ("(%s)"):format(configuration)

              -- TODO: Get available simulator from daemon and targets value should be auto generated based on results
              local display = ("%s %s %s"):format(command, target, config)

              local item = {
                command = "Watch",
                config = {
                  target = target,
                  command = command,
                  configuration = configuration,
                  value = display,
                  device = nil, -- reserverd later for run command
                },
              }

              if watch.is_watching(item.config, command) then
                item.ops = "Stop"
                item.value = display
                item.kind = command
              else
                item.ops = "Start"
                item.value = display
                item.kind = command
              end

              results[#results + 1] = item
            end
          end
        end

        return results
      end)(),
      entry_maker = function(entry)
        entry.ordinal = entry.value
        entry.display = function(e)
          local maker = maker {
            separator = " ",
            hl_chars = { ["|"] = "TelescopeResultsNumber" },
            items = { { width = 40 } },
          }

          return maker {
            { e.value, "TelescopeResultsMethod" },
          }
        end
        return entry
      end,
    },
    attach_mappings = function(_, _)
      a.select_default:replace(handle_action)
      return true
    end,
  }):find()
end

M.build_run = function(command, opts)
  opts = require("telescope.themes").get_dropdown(opts or {})
  local targets, configurations = get_current_dir_targets()

  picker(opts, {
    sorter = sorter {},
    prompt_title = command,
    finder = finder {
      results = (function()
        local results = {}
        --- TODO(nvim): Make nested picker based on available commands
        for _, target in ipairs(targets) do
          for _, configuration in ipairs(configurations) do
            local config = configuration == "Debug" and "" or ("(%s)"):format(configuration)

            -- TODO: Get available simulator from daemon and targets value should be auto generated based on results
            local display = ("%s %s"):format(target, config)

            results[#results + 1] = {
              command = command,
              target = target,
              configuration = configuration,
              value = display,
              device = nil, -- reserverd later for run command
            }
          end
        end

        return results
      end)(),
      entry_maker = function(entry)
        entry.ordinal = entry.value

        entry.display = function(e)
          local maker = maker {
            separator = " ",
            hl_chars = { ["|"] = "TelescopeResultsNumber" },
            items = { { width = 40 } },
          }

          return maker { { e.value, "TelescopeResultsMethod" } }
        end

        return entry
      end,
    },
    attach_mappings = function(_, _)
      a.select_default:replace(handle_action)
      return true
    end,
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
          local maker = maker {
            separator = " ",
            hl_chars = { ["|"] = "TelescopeResultsNumber" },
            items = { { width = 40 } },
          }

          return maker {
            { e.value, "TelescopeResultsMethod" },
          }
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
        elseif selected.value == "Build" or selected.value == "Run" then
          M.build_run(selected.value, opts)
        end
      end)
      return true
    end,
  }):find()
end

return M
