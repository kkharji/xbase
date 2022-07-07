---@type ffilib
local ffi = require "ffi"
local validate = vim.validate

---@class XBase
local M = {}

local RegisterStatus = {
  Registered = 0,
  NotSupported = 1,
  BroadcastSetupErrored = 2,
  ServerErrored = 3,
}

local native = require("ffi").load((function()
  local source = debug.getinfo(1).source
  local build_path = ("%s../../build"):format(source:sub(2, #"/server.lua" * -1))
  local library_path = build_path .. "/libxbase.so"
  local header_path = build_path .. "/libxbase.h"
  local header_lines = {}

  for line in io.lines(header_path) do
    header_lines[#header_lines + 1] = line
  end

  ffi.cdef(table.concat(header_lines))

  return library_path
end)())

---Register given root and return true if the root is registered
---@param root string
---@return boolean
function M.register(root)
  local root_ptr = ffi.new("char[?]", #root + 1, root)
  validate { root = { root, "string", false } }

  require("xbase.log").setup()
  -- TODO: schedule wrap?

  ---@type xbase.register.response
  local response = native.xbase_register(root_ptr)
  local status = response.status

  if RegisterStatus.NotSupported == status then
    return false
  elseif RegisterStatus.BroadcastSetupErrored == status then
    error "Registration failed, broadcast listener setup failed: Open Issue"
  elseif RegisterStatus.ServerErrored == status then
    error "Registration failed, Server Errored: Open Issue"
  end

  local parts = vim.split(root, "/")
  local name = parts[#parts]:gsub("^%l", string.upper)

  vim.notify(string.format("[%s] Connected ", name))

  -- TODO: Should pipe be tracked and closed?

  local pipe = vim.loop.new_pipe()
  pipe:open(response.fd)
  pipe:read_start(require "xbase.broadcast")
  return true
end

return M
