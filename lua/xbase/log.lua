local util = require "xbase.util"
local M = { bufnr = nil }

function M.toggle(vsplit)
  local bufnr = M.bufnr
  local win = vim.fn.win_findbuf(bufnr)[1]
  local cmd = vsplit and "vert sbuffer" or "sbuffer"
  local open = string.format("%s %s", cmd, bufnr)

  if win then
    vim.api.nvim_win_close(win, false)
  end

  vim.cmd(open)

  -- TODO: make height and width configurable
  if not vsplit then
    vim.api.nvim_win_set_height(0, 20)
  else
  end

  local mappings = require("xbase.config").values.mappings

  util.try_map(mappings.toggle_vsplit_log_buffer, function()
    vim.cmd "close"
    M.toggle(true)
  end)

  util.try_map(mappings.toggle_split_log_buffer, function()
    vim.cmd "close"
    M.toggle(false)
  end)

  vim.keymap.set("n", "q", "close", { buffer = true })

  vim.cmd "call feedkeys('G')"
end

function M.log(msg, level)
  local user_level = require("xbase.config").values.log_level
  if #msg == 0 or user_level > level then
    return
  end

  local line_count = vim.api.nvim_buf_line_count(M.bufnr)
  local line_first = vim.api.nvim_buf_get_lines(M.bufnr, 0, 1, false)[1]
  local row = (line_count == 1 and #line_first == 0) and 0 or -1

  vim.api.nvim_buf_set_lines(M.bufnr, row, -1, false, { msg })

  M.update_cursor_position(line_count)
end

function M.update_cursor_position(line_count)
  local winid, is_focused = M.window()

  -- Window is not open, don't do anything
  if not winid then
    return
  end

  if not is_focused then
    vim.api.nvim_win_set_cursor(winid, { line_count + 1, 0 })
    -- vim.fn.win_execute(winid, "call feedkeys('zt')", false)
    return
  end

  local diff = line_count - vim.api.nvim_win_get_cursor(winid)[1]

  if diff <= 2 then
    vim.api.nvim_win_set_cursor(winid, { line_count + 1, 0 })
    -- TODO: make this behavior configurable
    -- vim.fn.win_execute(winid, "call feedkeys('zt')", false)
  end
end

--- Get Log buffer window (if open) and whether it is focused or not
function M.window()
  local windows = vim.api.nvim_list_wins()

  for _, winid in ipairs(windows) do
    local win_bufnr = vim.api.nvim_win_get_buf(winid)
    if win_bufnr == M.bufnr then
      local curr_win = vim.api.nvim_get_current_win()
      local is_focused = curr_win == winid
      return winid, is_focused
    end
  end
  return nil, nil
end

function M.setup()
  local bufnr = vim.api.nvim_create_buf(false, true)

  vim.api.nvim_buf_set_name(bufnr, "[XBase Logs]")
  vim.api.nvim_buf_set_option(bufnr, "filetype", "xclog")
  vim.api.nvim_create_autocmd({ "BufEnter" }, {
    buffer = bufnr,
    -- TODO: make scrolloff configurable
    command = "setlocal nonumber norelativenumber scrolloff=4",
  })
  M.bufnr = bufnr
  return bufnr
end

return M
