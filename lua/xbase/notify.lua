local M = {}
local notify = function(msg, level)
  if #vim.trim(msg) == 0 then
    return
  end
  if level == 5 then
    vim.api.nvim_echo({ { msg, "healthSuccess" } }, true, {})
    return
  end

  vim.notify(msg, level, {
    title = "XBase",
  })
end

M = setmetatable(M, {
  __index = function(_, level)
    level = vim.log.levels[string.upper(level)]
    return function(msg)
      return notify(msg, level)
    end
  end,
  __call = function(_, msg, level)
    return notify(msg, vim.log.levels[level:upper()] or 5)
  end,
})

return M
