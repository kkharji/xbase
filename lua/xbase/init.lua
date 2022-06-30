local M = {}

M.lib = require "xbase_client"
M.targets = M.lib.targets
M.runners = M.lib.runners
M.watching = M.lib.watching
M.drop = M.lib.drop
M.build = M.lib.build
M.run = M.lib.run
local util = require "xbase.util"

M.toggle_log_buffer = function(vsplit)
  local bufnr = M.lib.log_bufnr()
  local win = vim.fn.win_findbuf(bufnr)[1]
  local cmd = vsplit and "vert sbuffer" or "sbuffer"
  local open = string.format("%s %s", cmd, bufnr)

  if win then
    vim.api.nvim_win_close(win, false)
  end

  vim.cmd(open)

  local mappings = require("xbase.config").values.mappings

  util.try_map(mappings.toggle_vsplit_log_buffer, function()
    vim.cmd "close"
  end)

  util.try_map(mappings.toggle_split_log_buffer, function()
    vim.cmd "close"
  end)

  vim.keymap.set("n", "q", "close", { buffer = true })
end

M.setup = function(opts)
  vim.schedule(function()
    opts = opts or {}
    local root = vim.loop.cwd()
    local config = require "xbase.config"
    config.set(opts)
    local config = config.values

    -- TODO: run try register on switch directories
    if M.lib.register(root) then
      vim.cmd [[ autocmd VimLeavePre * lua require'xbase'.drop()]]
      if config.mappings.enable then
        util.bind(config)
      end
    end

    vim.api.nvim_create_autocmd({ "BufEnter", "BufWinEnter" }, {
      pattern = { "*.m", "*.swift", "*.c", "*.yml" },
      callback = function()
        if config.mappings.enable then
          util.bind(config)
        end
      end,
    })
  end)
end

return M
