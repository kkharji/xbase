local config = require("xbase.config").values

local M = {}

function M.is_watching(config, command, device, watchlist)
  local root = vim.loop.cwd()
  local base_key = string.format("-configuration %s", config.configuration)
  local key

  if command == "Run" then
    if device then
      key = string.format("%s:%s:%s:%s", root, command, device.name, base_key)
    else
      key = string.format("%s:%s:%s:%s", root, command, "Bin", base_key)
    end
  else
    key = string.format("%s:%s:%s", root, command, base_key)
  end

  if config.sysroot then
    key = key .. " -sysroot " .. config.sysroot
  end

  if config.scheme then
    key = key .. " -scheme " .. config.scheme
  end

  key = key .. " -target " .. config.target

  for _, watching_key in pairs(watchlist) do
    if key == watching_key then
      return true
    end
  end

  return false
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
    require("xbase.log").toggle(false, true)
  end, bufnr)

  M.try_map(m.toggle_vsplit_log_buffer, function()
    require("xbase.log").toggle(true, true)
  end, bufnr)
end

function M.feline_provider()
  return {
    provider = function(_)
      icon = {}
      --- TODO(nvim): only show build status in xcode supported files?
      local config = config.statusline
      local status = vim.g.xbase_watch_build_status

      if status == "processing" then
        icon.str = config.running.icon
        icon.hl = { fg = config.running.color }
      elseif status == "running" then
        icon.str = config.device_running.icon
        icon.hl = { fg = config.device_running.color }
      elseif status == "success" then
        icon.str = config.success.icon
        icon.hl = { fg = config.success.color }
      elseif status == "failure" then
        icon.str = config.failure.icon
        icon.hl = { fg = config.failure.color }
      elseif status == "watching" then
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
