local config = require("xbase.config").values
local M = {}
---@param platform Platform
local get_devices = function(platform)
  local devices = {}

  if platform then
    for _, device in pairs(vim.g.xbase.devices) do
      if platform == device.platform then
        table.insert(devices, {
          name = device.info.name,
          udid = device.info.udid,
          is_on = device.info.state ~= "Shutdown",
        })
      end
    end
  end

  return devices
end

---Get Targets from project
---To Support Multi Platform targets
---@param project Project
M.get_targets_runners = function(project)
  local targets = {}

  for name, platform in pairs(project.targets) do
    table.insert(targets, {
      name = name,
      runners = get_devices(platform),
    })
  end

  return targets
end

M.reload_lsp_servers = function()
  -- local clients = require("lspconfig.util").get_managed_clients()
  -- local ids = ""
  -- for _, client in ipairs(clients) do
  --   if client.name == "sourcekit" then
  --     ids = ids .. client.id
  --   end
  -- end

  print "Restarting Servers"
  vim.cmd "LspRestart"
end

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
    I(key, watching_key)
    if key == watching_key then
      return true
    end
  end

  return false
end

M.feline_provider = function()
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
