local config = require("xbase.config").values

local M = {}

function M.is_watching(settings, command, device, watchlist)
  local root = vim.loop.cwd()
  local key = ("%s:%s"):format(root, command)

  if command == "Run" then
    key = key .. (device ~= nil and device.name or ":Bin")
  end

  key = key .. "-configuration " .. settings.configuration
  key = key .. " -target " .. settings.target

  return vim.tbl_contains(watchlist, key)
end

function M.project_name(root)
  local parts = vim.split(root, "/")
  return parts[#parts]:gsub("^%l", string.upper)
end

function M.try_map(key, fun, bufnr)
  if type(key) == "string" then
    vim.keymap.set("n", key, fun, { buffer = bufnr and bufnr or true })
  end
end

function M.bind(m, bufnr)
  local pickers = require "xbase.pickers"
  M.try_map(m.build_picker, pickers.build, bufnr)
  M.try_map(m.run_picker, pickers.run, bufnr)
  M.try_map(m.watch_picker, pickers.watch, bufnr)
  M.try_map(m.all_picker, pickers.actions, bufnr)
  M.try_map(m.toggle_split_log_buffer, function()
    require("xbase.logger").toggle(false, true)
  end, bufnr)

  M.try_map(m.toggle_vsplit_log_buffer, function()
    require("xbase.logger").toggle(true, true)
  end, bufnr)
end

function M.feline_provider()
  return {
    provider = function(_)
      icon = {}
      --- TODO(nvim): only show build status in xcode supported files?
      local config = config.statusline
      local status = vim.g.xbase_watch_build_status

      if status == "Processing" then
        icon.str = config.running.icon
        icon.hl = { fg = config.running.color }
      elseif status == "Running" then
        icon.str = config.device_running.icon
        icon.hl = { fg = config.device_running.color }
      elseif status == "Success" then
        icon.str = config.success.icon
        icon.hl = { fg = config.success.color }
      elseif status == "Failure" then
        icon.str = config.failure.icon
        icon.hl = { fg = config.failure.color }
      elseif status == "Watching" then
        icon.str = config.watching.icon
        icon.hl = { fg = config.watching.color }
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
