--Types

---@class XBaseSelectOptions
---@field root string? the project root to run against (default current root)

local ok, _ = pcall(require, "telescope")
if ok then
  return require "xbase.pickers.telescope"
else
  return require "xbase.pickers.builtin"
end
