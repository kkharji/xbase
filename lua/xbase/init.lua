local initialized = false
local server = require "xbase.server"
local util = require "xbase.util"
local config = require "xbase.config"
local autocmd = vim.api.nvim_create_autocmd

local function try_attach_mappings()
  if config.values.mappings.enable then
    util.bind(config.values.mappings)
  end
end

local function try_attach(root)
  local file_patterns = { "*.m", "*.swift", "*.c", "*.yml" }
  if server.should_register(root) then
    server.register(root)
    if not initialized then
      initialized = true
      autocmd({ "VimLeavePre" }, {
        pattern = "*",
        callback = function()
          server.drop(server.roots)
        end,
      })
      autocmd({ "BufEnter", "BufWinEnter" }, { pattern = file_patterns, callback = try_attach_mappings })
      autocmd({ "BufEnter" }, { pattern = "xclog", callback = try_attach_mappings })
    end
    try_attach_mappings()
  end
end

return {
  setup = function(opts)
    vim.schedule(function()
      opts = opts or {}
      config.set(opts)
      try_attach(vim.loop.cwd())
      autocmd({ "DirChanged" }, {
        pattern = "*",
        callback = function()
          try_attach(vim.loop.cwd())
        end,
      })
    end)
  end,
}
