---@type ffilib
local ffi = require "ffi"

---@class XBase
---@field xbase_hello function
local M = {}

local library_path = (function()
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
end)()

local native = require("ffi").load(library_path)

local root = "/lua/xbase/server.lua"
local c_str = ffi.new("char[?]", #root + 1, root)
-- ffi.copy(c_str, root)
local output = native.xbase_register(c_str)

I(output)

--         local pipe = vim.loop.new_pipe()
--         pipe:open(res.fd)
--         pipe:read_start(function(err, chunk)
--             assert(not err, err)
--             if chunk then
--                 vim.schedule(function()
--                      let message = M.parse_brodcast_message(chunk)
--                      $callback(chunk)
--                  end)
--             end
--         end)
return M
