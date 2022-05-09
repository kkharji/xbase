local lib = require "libxcodebase"
local config = require("xcodebase.config").values
local M = {}

M.start = function(opts)
  lib.watch_start(opts)
end

M.stop = function()
  lib.watch_stop {}
end

M.is_watching = false

M.feline_provider = function()
  return {
    provider = function(_)
      --- TODO(nvim): only show build status in xcode supported files?
      local config = config.statusline

      if not M.is_watching then
        return " ", {}
      end

      icon = {}
      local status = vim.g.xcodebase_watch_build_status
      if status == "running" then
        icon.str = config.running.icon
        icon.hl = { fg = config.running.color }
      elseif status == "success" then
        icon.str = config.success.icon
        icon.hl = { fg = config.success.color }
      elseif status == "failure" then
        icon.str = config.failure.icon
        icon.hl = { fg = config.failure.color }
      else
        icon.str = " "
      end

      if icon.str == " " then
        return " ", icon
      else
        icon.str = " [" .. icon.str .. " xcode]"
        return " ", icon
      end
    end,

    hl = {},
  }
end

return M

-- function M.file_info(component, opts)
--     local readonly_str, modified_str, icon

--     -- Avoid loading nvim-web-devicons if an icon is provided already
--     if not component.icon then
--         local icon_str, icon_color = require('nvim-web-devicons').get_icon_color(
--             fn.expand('%:t'),
--             nil, -- extension is already computed by nvim-web-devicons
--             { default = true }
--         )

--         icon = { str = icon_str }

--         if opts.colored_icon == nil or opts.colored_icon then
--             icon.hl = { fg = icon_color }
--         end
--     end

--     local filename = api.nvim_buf_get_name(0)
--     local type = opts.type or 'base-only'
--     if filename == '' then
--         filename = '[No Name]'
--     elseif type == 'short-path' then
--         filename = fn.pathshorten(filename)
--     elseif type == 'base-only' then
--         filename = fn.fnamemodify(filename, ':t')
--     elseif type == 'relative' then
--         filename = fn.fnamemodify(filename, ':~:.')
--     elseif type == 'relative-short' then
--         filename = fn.pathshorten(fn.fnamemodify(filename, ':~:.'))
--     elseif type == 'unique' then
--         filename = get_unique_filename(filename)
--     elseif type == 'unique-short' then
--         filename = get_unique_filename(filename, true)
--     elseif type ~= 'full-path' then
--         filename = fn.fnamemodify(filename, ':t')
--     end

--     if bo.readonly then
--         readonly_str = opts.file_readonly_icon or '🔒'
--     else
--         readonly_str = ''
--     end

--     -- Add a space at the beginning of the provider if there is an icon
--     if (icon and icon ~= '') or (component.icon and component.icon ~= '') then
--         readonly_str = ' ' .. readonly_str
--     end

--     if bo.modified then
--         modified_str = opts.file_modified_icon or '●'

--         if modified_str ~= '' then
--             modified_str = ' ' .. modified_str
--         end
--     else
--         modified_str = ''
--     end

--     return string.format('%s%s%s', readonly_str, filename, modified_str), icon
-- end
