local logger = require "xbase.log"
local notify = require "xbase.notify"
local socket = require "xbase.socket"
local M = {}

local Task = {
  OpenLogger = "OpenLogger",
  ReloadLspServer = "ReloadLspServer",
  UpdateStatusline = "UpdateStatusline",
}

local MessageType = {
  Log = "Log",
  Notify = "Notify",
  Execute = "Execute",
}

local handlers = {
  [MessageType.Log] = function(item)
    if #item.msg > 0 then
      logger.log(item.msg, item.level)
    end
  end,
  [MessageType.Notify] = function(item)
    notify(item.msg, item.level)
  end,
  [MessageType.Execute] = function(item)
    local task = item.task
    if Task.UpdateStatusline == task then
      vim.g.xbase_watch_build_status = item.value
    elseif Task.OpenLogger == task then
      logger.toggle(nil, false)
    elseif Task.ReloadLspServer == task then
      vim.cmd "LspRestart"
    end
  end,
}

function M.start(address)
  local socket = socket:connect(address)

  socket:read_start(function(chunk)
    local chunk = vim.trim(chunk)
    for _, chunk in ipairs(vim.split(chunk, "\n")) do
      local item = vim.json.decode(chunk)
      vim.schedule(function()
        handlers[item.type](item)
      end)
    end
  end)

  return socket
end

return M
