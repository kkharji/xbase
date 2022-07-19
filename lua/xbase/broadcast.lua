local logger = require "xbase.logger"
local notify = require "xbase.notify"
local socket = require "xbase.socket"
local types = require "xbase.types"
local msg, tkind, tstatus = types.Message, types.TaskKind, types.TaskStatus
local config = require("xbase.config").values

local M = {}

M.expect_second_run = false

local function task_set(args)
  M.has_task = true
  local running, _ = tkind:prefix(args.kind)
  args.prefix = running
  vim.g.xbase_ctask = args
  local line = string.format("[%s] %s", args.target, args.prefix)
  if tkind.is_run(args.kind) then
    vim.g.xbase_ctask_line = string.format("%s %s", config.statusline.device_running.icon, line)
  else
    vim.g.xbase_ctask_line = line
  end
  vim.schedule(require("xbase.statusline").update_spinner)
end

local function task_update(args)
  local content, level = args.content, args.level
  if #content == 0 then
    return
  end
  local ctask = vim.g.xbase_ctask
  local target, prefix, kind = ctask.target, ctask.prefix, ctask.kind

  logger.log(content, level)

  if level ~= "Debug" and level ~= "Trace" then
    content = string.gsub(content, "%[" .. target .. "%]%s", "")
    local line = string.format("[%s] %s: %s", target, prefix, content)

    if tkind.is_run(kind) then
      local icon = config.statusline.device_running.icon
      line = ("%s %s"):format(icon, line)
      vim.g.xbase_ctask_display = line
    end

    vim.g.xbase_ctask_line = line
  end
end

local function task_finish(args)
  M.has_task = false
  vim.g.xbase_ctask = vim.tbl_extend("force", vim.g.xbase_ctask, args)
  local line, level, icon
  local ctask = vim.g.xbase_ctask
  local target, prefix = ctask.target, ctask.prefix
  local is_failed = tstatus.is_failed(args.status)

  if is_failed then
    icon = config.statusline.failure.icon
    level = "Error"
  else
    icon = config.statusline.success.icon
    level = "Info"
  end

  if tkind.is_run(vim.g.xbase_ctask.kind) then
    vim.g.xbase_ctask_display = ("%s [%s] Device Disconnected"):format(icon, target)
  else
    if is_failed then
      line = ("[%s] %s Failed"):format(target, prefix)
    else
      local _, done = tkind:prefix(ctask.kind)
      line = ("[%s] %s"):format(target, done)
    end
    vim.g.xbase_ctask_line = ("%s %s"):format(icon, line)
  end

  vim.schedule(function()
    if line then
      logger.log(line, level)
    end
    vim.defer_fn(function()
      if M.has_task == false then
        vim.g.xbase_ctask_display = nil
        vim.g.xbase_ctask_line = nil
      end
    end, 5000)
  end)
end

function M.start(root, address)
  local socket = socket:connect(address)

  socket._socket:write(string.format("%s\n", vim.loop.os_getpid()), function(err)
    if err then
      print(socket._stream_error or err)
    end
  end)

  socket:read_start(function(chunk)
    local chunk = vim.trim(chunk)
    for _, chunk in ipairs(vim.split(chunk, "\n")) do
      local item = vim.json.decode(chunk)
      local type, args = item.type, item.args

      vim.schedule(function()
        if msg.task_is_update_current(type) then
          return task_update(args)
        end

        if msg.task_is_set_current(type) then
          return task_set(args)
        end

        if msg.task_is_finish_current(type) then
          return task_finish(args)
        end

        if msg.is_notify(type) then
          notify(args.content, args.level)
          if string.find(args.content, "Registered") ~= nil then
            vim.schedule(function()
              vim.defer_fn(function()
                print "  "
              end, 2000)
            end)
          end
          return
        end

        if msg.is_reload_lsp_server(type) then
          return vim.cmd "LspRestart"
        end

        if msg.is_open_logger(type) then
          return logger.toggle(nil, false)
        end

        if msg.is_log(type) then
          return logger.log(args.content, args.level)
        end

        if msg.is_set_state(type) then
          local key, value = args.key, args.value
          if key == "runners" then
            require("xbase.state").runners = value
          elseif key == "projectInfo" then
            require("xbase.state").project_info[root] = value
          end
          return
        end

        ---@diagnostic disable-next-line: empty-block
        if msg.is_set_watching(type) then
          -- ()
        end
      end)
    end
  end)

  return socket
end

return M
