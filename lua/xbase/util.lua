local M = {}

function M.project_name(root)
  local parts = vim.split(root, "/")
  return parts[#parts]:gsub("^%l", string.upper)
end

function M.try_map(key, fun, bufnr)
  if type(key) == "string" then
    vim.keymap.set("n", key, fun, { buffer = bufnr and bufnr or true })
  end
end

function M.bind(m, bufnr)
  local pickers = require "xbase.pickers"
  M.try_map(m.build_picker, pickers.build, bufnr)
  M.try_map(m.run_picker, pickers.run, bufnr)
  M.try_map(m.watch_picker, pickers.watch, bufnr)
  M.try_map(m.all_picker, pickers.actions, bufnr)
  M.try_map(m.toggle_split_log_buffer, function()
    require("xbase.logger").toggle(false, true)
  end, bufnr)

  M.try_map(m.toggle_vsplit_log_buffer, function()
    require("xbase.logger").toggle(true, true)
  end, bufnr)
end
--
--- Inserts all of the items in the second table into the first table
M.insert_all = function(base_table, new_items)
    for _, item in pairs(new_items) do
        table.insert(base_table, item)
    end
end



return M
