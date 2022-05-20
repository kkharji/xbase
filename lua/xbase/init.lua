local M = {}

vim.g.xbase = {
  ---@type Project[]
  projects = vim.empty_dict(),
  ---@type table<string, boolean>
  watch = vim.empty_dict(),
  ---@type Device[]
  devices = vim.empty_dict(),
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
M.register = function(root)
  local root = root or vim.loop.cwd()
  local _ = require("libxbase").ensure()
  require("libxbase").register(root)
end

---Tries to register vim instance as client for xbase server.
---Only register the vim instance when `xbase.should_attach`
---@see xbase.should_attach
M.try_register = function(root, opts)
  opts = opts or {}
  if M.should_register(root, opts) then
    M.register()
    vim.cmd [[ autocmd VimLeavePre * lua require'xbase'.drop(true)]]
  end
end

M.drop = function(remove_client)
  require("libxbase").drop { remove_client = remove_client }
end

M.build = function(opts)
  require("libxbase").build(opts)
end

M.run = function(opts)
  require("libxbase").run(opts)
end

M.watch = function(opts)
  require("libxbase").watch_target(opts)
end

---Setup xbase for current instance.
---Should ran once per neovim instance
---@param opts xbaseOptions
---@overload fun()
M.setup = function(opts)
  vim.schedule(function()
    opts = opts or {}
    local root = vim.loop.cwd()
    -- Mutate xbase configuration
    require("xbase.config").set(opts)
    -- Try to register current vim instance
    -- NOTE: Should this register again on cwd change?
    M.try_register(root, opts)

    vim.api.nvim_create_autocmd({ "BufEnter", "BufWinEnter" }, {
      pattern = { "*.m", "*.swift", "*.c", "*.yml" },
      callback = function()
        vim.keymap.set("n", "<leader>ef", require("xbase.pickers").actions, { buffer = true })
      end,
    })
    -- so that on a new buffer it would work
    vim.keymap.set("n", "<leader>ef", require("xbase.pickers").actions, { buffer = true })
  end)
end

return M
