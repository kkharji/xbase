local M = {}
local util = require "xbase.pickers.util"
local xbase = require "xbase.pickers.util"

local C = util.Command
do
  ---Given action create and run picker
  ---@param name XBaseCommand
  ---@param opts XBaseSelectOptions
  local find = function(name, opts)
    opts = opts or {}
    opts.root = opts.root or vim.loop.cwd()

    vim.ui.select(xbase.generate_entries(opts.root, name), {
      prompt = "XBase " .. name,
      format_item = function(entry)
        local e = xbase.prep_entry(entry)
        return e.ordinal
      end,
    }, function(entry)
      if not entry then
        return
      end
      xbase.run_command(entry)
    end)
  end

  ---XBase watch picker
  ---@param opts table
  M.watch = function(opts)
    find(C.Watch, opts)
  end

  ---XBase build picker
  M.build = function(opts)
    find(C.Build, opts)
  end

  ---XBase run picker
  M.run = function(opts)
    find(C.Run, opts)
  end
end

---Get all available xbase actions
---@param opts XBaseSelectOptions?
M.actions = function(opts)
  opts = opts or {}
  opts.root = opts.root or vim.loop.cwd()

  vim.ui.select(util.action_entries, {
    prompt = xbase.action_prompt,
    format_item = function(item)
      return item.value
    end,
  }, function(choice)
    if not choice then
      return
    end
    if choice.value == C.Watch then
      M.watch(opts)
    elseif choice.value == C.Build then
      M.build(opts)
    elseif choice.value == C.Run then
      M.run(opts)
    end
  end)
end

return M
