return function(msg, level)
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
