-- TODO: Generate this file out of rust structs
local M = {}

---@alias xbase.register_status
---| 0 # Project root is registered
---| 1 # Project root is not supported
---| 2 # Field to setup broadcast writer
---| 3 # Server errored

---@class xbase.register.response
---@field fd number @broadcast reader file description
---@field status xbase.register_status @broadcast reader file description
return M
