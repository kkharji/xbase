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

local function try_attach_code_actions()
	local has_null_ls = pcall(require, "null-ls")
	if not (has_null_ls and config.values.code_actions.enable) then
		return
	end
	local null_ls = require("null-ls")
    local swift_actions = require("xbase.treesitter")
    if swift_actions == nil then
        return
    end

	-- Deregister actions first because actions are duplicated when resourcing null-ls
	null_ls.deregister("xbase-treesitter-actions")
	null_ls.register({
		name = "xbase-treesitter-actions",
		method = { require("null-ls").methods.CODE_ACTION },
		filetypes = { "swift" },
		generator = {
			fn = function()
				return {
					-- Adds the .padding modifier to a view
					{
						title = "Modify padding",
						action = swift_actions.add_modifier("padding", ".top", 4)
						,
					},
					-- Adds the .font modifier to a view
					{
						title = "Modify font",
						action = swift_actions.add_modifier("font", ".headline")
					},
					{
						title = "Extract variable to struct field",
						action = swift_actions.extract_variable_to_struct
					},
					{
						title = "Extract to new view",
						action = swift_actions.extract_component
					},
				}
			end,
		},
	})
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
  try_attach_code_actions()
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
