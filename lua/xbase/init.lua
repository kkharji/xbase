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

local setup_lsp = function(opts)
  local ok, lspconfig = pcall(require, "lspconfig")

  if not ok then
    return
  end

  local extend, setup = vim.tbl_deep_extend, lspconfig.sourcekit.setup
  local pattern = require("lspconfig.util").root_pattern
  local root_mkr = pattern("Package.swift", ".git", "project.yml", "Project.swift")

  setup(extend("keep", opts.sourcekit, {
    cmd = { "sourcekit-lsp", "--log-level", "error" },
    filetypes = { "swift" },
    root_dir = root_mkr,
  }))
end

local function try_attach(root, opts)
  local file_patterns = { "*.m", "*.swift", "*.c", "*.yml" }

  if server.should_register(root) then
    server.register(root)
    if not initialized then
      initialized = true
      if opts.sourcekit ~= nil then
        setup_lsp(opts)
      end
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
      try_attach(vim.loop.cwd(), config.values)
      autocmd({ "DirChanged" }, {
        pattern = "*",
        callback = function()
          try_attach(vim.loop.cwd())
        end,
      })
    end)
  end,
}
