local xcodebase = {}
local config = require "xcodebase.config"
local lib = require "libxcodebase"

---@class XcodeBaseServer
---@field ensure fun():boolean: When the a new server started it should return true
---@field is_running fun():boolean
---@field register fun(pid: number, root: string):boolean
local server = lib.server

---@class XcodeBaseCommand
local command = lib.command

---@class XcodeBaseService
local command = lib.service

---Check whether the vim instance should be registered to xcodebase server.
---NOTE: Only support project.yml
---@param root string: current working directory
---@param _ table: options to influence the result.
---@return boolean
xcodebase.should_register = function(root, _)
  if vim.loop.fs_stat(root .. "/project.yml") then
    return true
  end

  return false
end

---Tries to register vim instance as client for xcodebase server.
---Only register the vim instance when `xcodebase.should_attach`
---@see xcodebase.should_attach
xcodebase.try_register = function(opts)
  opts = opts or {}
  local root = vim.loop.cwd()

  if xcodebase.should_register(root, opts) then
    local _ = server.ensure()
    server.register(vim.fn.getpid(), root)
  else
    return
  end
end

---Setup xcodebase for current instance.
---Should ran once per neovim instance
---@param opts XcodeBaseOptions
---@overload fun()
xcodebase.setup = function(opts)
  opts = opts or {}

  -- Mutate xcodebase configuration
  config.set(opts)

  -- Try to register current vim instance
  -- NOTE: Should this register again on cwd change?
  xcodebase.try_register(opts)
end

return xcodebase
