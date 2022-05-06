local M = {}

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
