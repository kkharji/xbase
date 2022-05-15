local M = {}
local config = require "xbase.config"
local lib = require "libxbase"

vim.g.xbase = {
  projects = vim.empty_dict(),
  watch = vim.empty_dict(),
}

---Check whether the vim instance should be registered to xbase server.
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

---Tries to register vim instance as client for xbase server.
---Only register the vim instance when `xbase.should_attach`
---@see xbase.should_attach
M.try_register = function(root, opts)
  opts = opts or {}
  if M.should_register(root, opts) then
    M.register()
    vim.cmd [[ autocmd VimLeavePre * lua require'xbase'.drop()]]
  end
end

M.drop = function()
  lib.drop {
    remove_client = true,
  }
end

M.build = function(opts)
  lib.build(opts)
end

M.watch = function(opts)
  lib.watch_target(opts)
end

---Setup xbase for current instance.
---Should ran once per neovim instance
---@param opts xbaseOptions
---@overload fun()
M.setup = function(opts)
  local root = vim.loop.cwd()
  opts = opts or {}
  -- Mutate xbase configuration
  config.set(opts)
  -- Try to register current vim instance
  -- NOTE: Should this register again on cwd change?
  M.try_register(root, opts)

  vim.api.nvim_create_autocmd({ "BufEnter", "BufWinEnter" }, {
    pattern = { "*.m", "*.swift", "*.c" },
    callback = function()
      vim.keymap.set("n", "<leader>ef", require("xbase.pickers").actions, { buffer = true })
    end,
  })
  -- so that on a new buffer it would work
  vim.keymap.set("n", "<leader>ef", require("xbase.pickers").actions, { buffer = true })
end

return M
