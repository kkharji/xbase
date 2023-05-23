local have_null_ls, null_ls = pcall(require, "null-ls")

if not have_null_ls then
    return nil
end

return function(code_actions)
	-- Deregister actions first because actions are duplicated when resourcing null-ls
	null_ls.deregister("xbase-treesitter-actions")
	null_ls.register({
		name = "xbase-treesitter-actions",
		method = { require("null-ls").methods.CODE_ACTION },
		filetypes = { "swift" },
		generator = {
			fn = function()
                return code_actions
            end,
		},
	})
end
