local M = {}
local a = require "telescope.actions"
local s = require "telescope.actions.state"
local finder = require("telescope.finders").new_table
local picker = require("telescope.pickers").new
local sorter = require("telescope.config").values.generic_sorter
local maker = require("telescope.pickers.entry_display").create
local xcodebase = require "xcodebase"
local watch = require "xcodebase.watch"

--[[
-- Run <Simulator>
-- Watch Run <Simulator>
-- Run <Device>
-- Watch Build Debug 
-- Watch Build Release
--]]
--
M.all_actions = function(opts)
  opts = require("telescope.themes").get_dropdown(opts or {})
  local root = vim.loop.cwd()
  local info = vim.g.xcodebase.projects[root]
  if info == nil then
    error "No info available"
    -- info = xcodebase.project_info(root)
  end

  local targets = {}

  -- TOOD(core): Support custom schemes
  for name, _ in pairs(info.targets) do
    targets[#targets + 1] = name
  end

  local commands = { "Build", "Run" }

  -- TOOD(core): Support custom project configurations
  local configurations = { "Debug", "Release" }

  local command_plate = {}

  --- TODO(nvim): Make nested picker based on available commands

  for _, target in ipairs(targets) do
    for _, command in ipairs(commands) do
      for _, configuration in ipairs(configurations) do
        -- TODO: Get available simulator from daemon and targets
        -- value should be auto generated based on results
        local display = ("%s %s %s"):format(
          command,
          target,
          configuration == "Debug" and "" or ("(%s)"):format(configuration)
        )

        local item = {
          target = target,
          command = command,
          configuration = configuration,
          value = display,
          device = nil, -- reserverd later for run command
        }

        command_plate[#command_plate + 1] = item

        if watch.is_watching(item) then
          local stop = vim.deepcopy(item)
          stop.value = "Stop Watch " .. stop.value
          stop.command = "WatchStop"
          command_plate[#command_plate + 1] = stop
        else
          local start = vim.deepcopy(item)
          start.value = "Watch " .. start.value
          start.command = "Watch"
          command_plate[#command_plate + 1] = start
        end
      end
    end
  end

  picker(opts, {
    sorter = sorter {},
    prompt_title = "Execute action",
    finder = finder {
      results = command_plate,
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
        if selected.command == "Build" then
          xcodebase.build(selected)
        elseif selected.command == "Watch" then
          watch.start(selected)
        elseif selected.command == "WatchStop" then
          watch.stop(selected)
        end
      end)
      return true
    end,
  }):find()
end

return M
