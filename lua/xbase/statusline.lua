local statusline_cfg = require("xbase.config").values.statusline
local types = require "xbase.types"
local tkind, tstatus = types.TaskKind, types.TaskStatus

local M = { highlight = {}, spinner_idx = 0 }

M.spinner = { "⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏" }

local function get_fg_from_group(name)
  if M.highlight[name] then
    return M.highlight[name]
  else
    local value = vim.fn.synIDattr(vim.fn.synIDtrans(vim.fn.hlID(name)), "fg")
    M.highlight[name] = value
    return value
  end
end

function M.should_spin()
  return tstatus.is_processing(vim.g.xbase_ctask.status) and (not tkind.is_run(vim.g.xbase_ctask.kind))
end

function M.update_spinner()
  if M.should_spin() then
    M.spinner_idx = M.spinner_idx + 1
    if M.spinner[M.spinner_idx] == nil then
      M.spinner_idx = 1
    end

    vim.g.xbase_ctask_display = ("%s %s"):format(M.spinner[M.spinner_idx], vim.g.xbase_ctask_line)

    vim.defer_fn(function()
      if M.should_spin() then
        M.update_spinner()
      else
        if tkind.is_run(vim.g.xbase_ctask.kind) then
          vim.g.xbase_ctask_display = vim.g.xbase_ctask_line
        else
          M.end_spinner()
        end
      end
    end, 100)
  else
    M.end_spinner()
  end
end

function M.end_spinner()
  vim.g.xbase_ctask_display = vim.g.xbase_ctask_line
  vim.defer_fn(function()
    if not tkind.is_run(vim.g.xbase_ctask.kind) and not require("xbase.broadcast").has_task then
      vim.g.xbase_ctask_display = ""
    end
  end, 5000)
end

function M.feline()
  return {
    provider = function(_)
      local data = {}
      if vim.g.xbase_ctask_display and vim.g.xbase_ctask_display:len() ~= 0 then
        data.str = " " .. vim.g.xbase_ctask_display
        --- TODO(nvim): only show build status in xcode supported files?
        local status = vim.g.xbase_ctask.status
        local kind = vim.g.xbase_ctask.kind

        if tstatus.is_succeeded(status) then
          data.hl = { fg = statusline_cfg.success.color }
        elseif tstatus.is_failed(status) then
          data.hl = { fg = statusline_cfg.failure.color }
        elseif tkind.is_run(kind) then
          data.hl = { fg = statusline_cfg.device_running.color }
        else
          data.hl = { fg = get_fg_from_group "Comment" }
        end

        return " ", data
      elseif vim.g.xbase_ctask_watching then
        data.hl = statusline_cfg.watching.color
        data.str = statusline_cfg.watching.icon
        -- TODO: Add watched target name
        return " ", data
      end
      return " ", {}
    end,

    hl = {},
  }
end

return M
