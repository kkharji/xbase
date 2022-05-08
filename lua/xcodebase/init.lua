local M = {}
local config = require "xcodebase.config"
local lib = require "libxcodebase"
local pid = vim.fn.getpid()

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

--- Register current neovim client
M.register = function()
  local _ = lib.ensure()
  lib.register { address = vim.env.NVIM_LISTEN_ADDRESS }
end

---Tries to register vim instance as client for xcodebase server.
---Only register the vim instance when `xcodebase.should_attach`
---@see xcodebase.should_attach
M.try_register = function(root, opts)
  opts = opts or {}
  if M.should_register(root, opts) then
    M.register()
    vim.cmd [[ autocmd VimLeavePre * lua require'xcodebase'.drop()]]
  end
end

M.drop = function()
  lib.drop {}
end

M.build = function(opts)
  local root = vim.loop.cwd()
  lib.build(vim.tbl_extend("keep", opts or {}, {
    pid = pid,
    root = root,
  }))
end

M.project_info = function(root)
  M.projects[root] = nil
  lib.project_info(pid, root)
  while M.projects[root] == nil do
  end
end

---Setup xcodebase for current instance.
---Should ran once per neovim instance
---@param opts XcodeBaseOptions
---@overload fun()
M.setup = function(opts)
  local root = vim.loop.cwd()
  opts = opts or {}
  -- Mutate xcodebase configuration
  config.set(opts)
  -- Try to register current vim instance
  -- NOTE: Should this register again on cwd change?
  M.try_register(root, opts)
end

---@class XcodeTarget
---@field type string
---@field platform string
---@field sources string[]

---@class XcodeProject
---@field name string @Project name
---@field targets string @Project name
---@field root string @Project root

---Holds project informations
---@type table<string, XcodeProject>
M.projects = {}

return M
