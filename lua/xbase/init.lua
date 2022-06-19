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
  elseif vim.loop.fs_stat(root .. "/Project.swift") then
    return true
    -- elseif vim.loop.fs_stat(root .. "/Package.swift") then
    -- return true
  elseif vim.fn.glob(root .. "/*.xcodeproj"):len() ~= 0 then
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
  I(opts)
  require("libxbase").run(opts)
end

local function try_map(key, fun)
  if type(key) == "string" then
    vim.keymap.set("n", key, fun, { buffer = true })
  end
end

M.toggle_log_buffer = function(vsplit)
  local bufnr = vim.g.xbase_log_bufnr
  local win = vim.fn.win_findbuf(bufnr)[1]

  if win then
    vim.api.nvim_win_close(win, false)
  end
  print "Opening log buffer"
  local cmd = vsplit and "vert sbuffer" or "sbuffer"
  local open = string.format("%s %s", cmd, bufnr)
  vim.cmd(open)

  local mappings = require("xbase.config").values.mappings

  try_map(mappings.toggle_vsplit_log_buffer, function()
    vim.cmd "close"
  end)

  try_map(mappings.toggle_split_log_buffer, function()
    vim.cmd "close"
  end)

  vim.keymap.set("n", "q", "close", { buffer = true })
end

local function bind(config)
  local pickers = require "xbase.pickers"

  if config.mappings.enable then
    try_map(config.mappings.build_picker, pickers.build)
    try_map(config.mappings.run_picker, pickers.run)
    try_map(config.mappings.watch_picker, pickers.watch)
    try_map(config.mappings.all_picker, pickers.actions)
    try_map(config.mappings.toggle_split_log_buffer, function()
      M.toggle_log_buffer(false)
    end)
    try_map(config.mappings.toggle_vsplit_log_buffer, function()
      M.toggle_log_buffer(true)
    end)
  end
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
    local config = require "xbase.config"

    config.set(opts)

    local config = config.values
    -- Try to register current vim instance
    -- NOTE: Should this register again on cwd change?
    M.try_register(root, opts)

    vim.api.nvim_create_autocmd({ "BufEnter", "BufWinEnter" }, {
      pattern = { "*.m", "*.swift", "*.c", "*.yml" },
      callback = function()
        bind(config)
      end,
    })
    bind(config)
  end)
end

return M
