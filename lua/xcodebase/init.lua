local M = {}
local config = require "xcodebase.config"
local lib = require "libxcodebase"
local pid = vim.fn.getpid()
local address = vim.env.NVIM_LISTEN_ADDRESS

---@class XcodeBaseDaemon
---@field ensure fun():boolean: When the a new server started it should return true
---@field is_running fun():boolean
---@field register fun(pid: number, root: string):boolean
M.daemon = lib.daemon

---@class XcodeBaseCommand
local command = lib.command

---@class XcodeBaseService
local command = lib.service

---Check whether the vim instance should be registered to xcodebase server.
---NOTE: Only support project.yml
---@param root string: current working directory
---@param _ table: options to influence the result.
---@return boolean
M.should_register = function(root, _)
  if vim.loop.fs_stat(root .. "/project.yml") then
    return true
  end

  return false
end

---Tries to register vim instance as client for xcodebase server.
---Only register the vim instance when `xcodebase.should_attach`
---@see xcodebase.should_attach
M.try_register = function(opts)
  opts = opts or {}
  local root = vim.loop.cwd()

  if M.should_register(root, opts) then
    local _ = M.daemon.ensure()
    M.daemon.register(pid, root, address)
    vim.cmd [[ autocmd VimLeavePre * lua require'xcodebase'.daemon.drop(vim.fn.getpid(), vim.loop.cwd())]]
  else
    return
  end
end

---Setup xcodebase for current instance.
---Should ran once per neovim instance
---@param opts XcodeBaseOptions
---@overload fun()
M.setup = function(opts)
  opts = opts or {}

  -- Mutate xcodebase configuration
  config.set(opts)

  -- Try to register current vim instance
  -- NOTE: Should this register again on cwd change?
  M.try_register(opts)
end

return M
