local M = {}

M.lib = require "xbase_client"
M.targets = M.lib.targets
M.runners = M.lib.runners
M.watching = M.lib.watching
M.drop = M.lib.drop
M.build = M.lib.build
M.run = M.lib.run
local util = require "xbase.util"
M.init = false

M.try_attach = function(root, config)
  if not M.lib.register(root) then
    return
  end
  if not M.init then
    M.init = true
    vim.api.nvim_create_autocmd({ "VimLeavePre" }, {
      pattern = "*",
      callback = function()
        M.drop()
      end,
    })

    vim.api.nvim_create_autocmd({ "BufEnter", "BufWinEnter" }, {
      pattern = { "*.m", "*.swift", "*.c", "*.yml" },
      callback = function()
        if config.mappings.enable then
          util.bind(config.mappings)
        end
      end,
    })

    vim.api.nvim_create_autocmd({ "BufEnter" }, {
      pattern = "xclog",
      callback = function()
        if config.mappings.enable then
          util.bind(config.mappings)
        end
      end,
    })
  end

  if config.mappings.enable then
    util.bind(config.mappings)
  end
end

M.setup = function(opts)
  vim.schedule(function()
    opts = opts or {}
    local root = vim.loop.cwd()
    local config = require "xbase.config"
    config.set(opts)
    local config = config.values

    M.try_attach(root, config)

    vim.api.nvim_create_autocmd({ "DirChanged" }, {
      pattern = "*",
      callback = function()
        M.try_attach(vim.loop.cwd(), config)
      end,
    })
  end)
end

return M
