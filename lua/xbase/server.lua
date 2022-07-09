local util = require "xbase.util"
local socket = require "xbase.socket"
local validate = vim.validate
local notify = require "xbase.notify"
local uv = vim.loop

---@class XBase
local M = {
  ---@type XBaseSocket @helper object to communcate with xbase daemon
  socket = nil,
  ---@type string[] @list of registered roots
  roots = {},
}

---Send Request to socket, and on response call on_response with data if no error
---@param req table
---@param on_response? function(response:table)
function M.request(req, on_response)
  -- TODO: ensure daemon is running!
  if not M.socket then
    M.socket = socket:connect()
  end

  -- TODO: what todo when we get nil?
  M.socket:read_start(function(chunk)
    if chunk ~= nil then
      vim.schedule(function()
        local res = vim.json.decode(chunk)
        if res.error then
          notify.error(string.format("%s %s", res.error.kind, res.error.msg))
          return
        end
        if on_response then
          on_response(res.data)
        end
      end)
    end
    M.socket:read_stop()
  end)

  M.socket:write(req)
end

---Check whether the vim instance should be registered to xbase server.
---@param root string: current working directory
---@return boolean
function M.should_register(root)
  if uv.fs_stat(root .. "/project.yml") then
    return true
  elseif uv.fs_stat(root .. "/Project.swift") then
    return true
  elseif uv.fs_stat(root .. "/Package.swift") then
    return true
  elseif vim.fn.glob(root .. "/*.xcodeproj"):len() ~= 0 then
    return true
  end
  return false
end

---Register given root and return true if the root is registered
---@param root string
---@return boolean
function M.register(root)
  validate { root = { root, "string", false } }

  require("xbase.log").setup()

  M.request({ method = "register", root = root }, function(broadcast_address)
    notify.info(("[%s] Connected "):format(util.project_name(root)))

    -- TODO: Should pipe be tracked and closed?
    -- TODO: ensure a connection is indeed made
    local broadcaster = socket:connect(broadcast_address)
    broadcaster:read_start(require "xbase.broadcast")

    M.roots[#M.roots + 1] = root

    -- Fill state with available runners
    if not require("xbase.state").runners then
      M.request({ method = "get_runners" }, function(runners)
        require("xbase.state").runners = runners
      end)
    end
  end)
end

---Get Project information
function M.get_project_info(root, on_response)
  M.request({
    method = "get_project_info",
    root = root,
  }, on_response)
end

---Drop a given root or drop all tracked roots if root is nil
---@param roots string|string[]
function M.drop(roots)
  M.request {
    method = "drop",
    roots = type(roots) == "string" and { roots } or roots,
  }
end

return M
