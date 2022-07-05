local util = require "xbase.util"
local M = { bufnr = nil }

function M.setup()
  if M.bufnr then
    return
  end
  M.bufnr = vim.api.nvim_create_buf(false, true)
  local cfg = require("xbase.config").values

  vim.api.nvim_buf_set_name(M.bufnr, "[XBase Logs]")
  vim.api.nvim_buf_set_option(M.bufnr, "filetype", "xclog")
  vim.api.nvim_create_autocmd({ "BufEnter" }, {
    buffer = M.bufnr,
    callback = function()
      vim.cmd "setlocal nonumber norelativenumber scrolloff=4"
    end,
  })

  util.bind(cfg.mappings, M.bufnr)
  vim.keymap.set("n", "q", "close", { buffer = M.bufnr })

  return M.bufnr
end

function M.toggle(vsplit, force)
  local cfg = require("xbase.config").values
  local curr_win = vim.api.nvim_get_current_win()

  local bufnr = M.bufnr
  local win = vim.fn.win_findbuf(bufnr)[1]

  if win and force then
    vim.api.nvim_win_close(win, false)
  end
  if win and not force then
    return
  end

  if vsplit == nil then
    local default = cfg.log_buffer.default_direction
    if default == "horizontal" then
      vsplit = false
    else
      vsplit = true
    end
  end

  local cmd = vsplit and "vert sbuffer" or "sbuffer"
  local open = string.format("%s %s", cmd, bufnr)

  vim.cmd(open)

  if vsplit == false then
    vim.api.nvim_win_set_height(0, cfg.log_buffer.height)
  else
    vim.api.nvim_win_set_width(0, cfg.log_buffer.width)
  end

  if not cfg.log_buffer.focus and curr_win ~= win then
    vim.api.nvim_set_current_win(curr_win)
  end
end

function M.log(msg, level)
  local config = require "xbase.config"
  config.set_log_level()
  local config = config.values

  if #msg == 0 or config.log_level > level then
    return
  end

  local line_count = vim.api.nvim_buf_line_count(M.bufnr)
  local line_first = vim.api.nvim_buf_get_lines(M.bufnr, 0, 1, false)[1]
  local row = (line_count == 1 and #line_first == 0) and 0 or -1

  vim.api.nvim_buf_set_lines(M.bufnr, row, -1, false, { msg })

  --- FIXME: Sometimes getting Error log.lua:89: Cursor position outside buffer
  -- Ignoring ..
  pcall(M.update_cursor_position, line_count)
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

return M
