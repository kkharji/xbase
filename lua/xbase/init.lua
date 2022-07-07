local server = require "xbase.server"
local util = require "xbase.util"

local M = {}
M.init = false

M.try_attach = function(root, config)
  if not server.register(root) then
    return
  end
  if not M.init then
    M.init = true
    vim.api.nvim_create_autocmd({ "VimLeavePre" }, {
      pattern = "*",
      callback = function()
        server.drop()
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

M.setup = function(_)
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
