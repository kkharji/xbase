local logger = require "xbase.logger"
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
  [MessageType.Log] = function(args)
    if #args.msg > 0 then
      logger.log(args.msg, args.level)
    end
  end,
  [MessageType.Notify] = function(args)
    notify(args.msg, args.level)
  end,
  [MessageType.Execute] = function(args)
    local task = args.task
    if Task.UpdateStatusline == task then
      vim.g.xbase_watch_build_status = args.value
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
        handlers[item.type](item.args)
      end)
    end
  end)

  return socket
end

return M
