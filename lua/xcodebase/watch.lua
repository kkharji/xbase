local lib = require "libxcodebase"
local config = require("xcodebase.config").values
local M = {}

M.start = function(opts)
  lib.watch_start { request = opts }
end

M.stop = function()
  lib.watch_stop {}
end

M.feline_provider = function()
  return {
    provider = function(_)
      --- TODO(nvim): only show build status in xcode supported files?
      local config = config.statusline

      -- if not M.is_watching then
      --   return " ", {}
      -- end

      icon = {}
      local status = vim.g.xcodebase_watch_build_status
      if status == "running" then
        icon.str = config.running.icon
        icon.hl = { fg = config.running.color }
      elseif status == "success" then
        icon.str = config.success.icon
        icon.hl = { fg = config.success.color }
      elseif status == "failure" then
        icon.str = config.failure.icon
        icon.hl = { fg = config.failure.color }
      else
        icon.str = " "
      end

      if icon.str == " " then
        return " ", icon
      else
        icon.str = " [" .. icon.str .. " xcode]"
        return " ", icon
      end
    end,

    hl = {},
  }
end

return M
