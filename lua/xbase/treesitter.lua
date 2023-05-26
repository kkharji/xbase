local have_treesitter = pcall(require, "nvim-treesitter")
if not have_treesitter then
	return nil
end

local ts_utils = require("nvim-treesitter.ts_utils")
local get_node_text = function(node)
	return vim.treesitter.get_node_text(node, vim.api.nvim_get_current_buf())
end
local get_winr = vim.api.nvim_get_current_win
local get_current_node = ts_utils.get_node_at_cursor

local query_parser = vim.fn.has("nvim-0.9") == 1 and vim.treesitter.query.parse or vim.treesitter.query.parse_query

local have_swift_parser, property_declaration_query = pcall(query_parser, "swift", "(property_declaration) @prop")

if not have_swift_parser then
	return
end

local find_next_call_expression = function(node)
	node = node:parent()
	while node ~= nil and node:type() ~= "call_expression" do
		node = node:parent()
	end

	return node
end
local get_entire_call_expression = function()
	local node = ts_utils.get_node_at_cursor()
	local call_expression = find_next_call_expression(node)
	while call_expression:parent():field("target")[1] ~= nil do
		call_expression = find_next_call_expression(call_expression)
	end

	return call_expression
end

local wrap_in_vstack = function()
	local bufnr = vim.api.nvim_get_current_buf()
	local call_expression = get_entire_call_expression()
	local range = { call_expression:range() }
	local indentation = string.rep(" ", range[2])
	local lines = vim.api.nvim_buf_get_lines(bufnr, range[1], range[3] + 1, false)
	local new_indented_lines = {}
	for _, line in pairs(lines) do
		line = "    " .. line
		table.insert(new_indented_lines, line)
	end
	vim.api.nvim_buf_set_lines(bufnr, range[1], range[3] + 1, false, new_indented_lines)
	vim.api.nvim_buf_set_lines(bufnr, range[3] + 1, range[3] + 1, false, { indentation .. "}" })
	vim.api.nvim_buf_set_lines(bufnr, range[1], range[1], false, { indentation .. "VStack {" })
	local winr = vim.api.nvim_get_current_win()
	vim.api.nvim_win_set_cursor(winr, { range[1] + 1, range[2] })
end

local create_modifier = function(modifier, arg1, arg2)
	arg1 = arg1 or ""
	print(modifier, arg1, arg2)
	local bufnr = vim.api.nvim_get_current_buf()
	local call_expression = get_entire_call_expression()
	local range = { call_expression:range() }
	local indentation = string.rep(" ", range[2])
	local padding_location
	if arg2 ~= nil then
		padding_location = string.format(".%s(%s, %s)", modifier, arg1, arg2)
	else
		padding_location = string.format(".%s(%s)", modifier, arg1)
	end
	vim.api.nvim_buf_set_lines(bufnr, range[3] + 1, range[3] + 1, false, { indentation .. padding_location })
	local winr = vim.api.nvim_get_current_win()
	vim.api.nvim_win_set_cursor(winr, { range[3] + 2, padding_location:len() + range[2] - 2 })
end

local add_modifier = function(modifier, arg1, arg2)
	return function()
		create_modifier(modifier, arg1, arg2)
	end
end

local get_bufnr = function()
	return vim.api.nvim_get_current_buf()
end

local get_enclosing_struct = function(node)
	node = node:parent()
	while node ~= nil and node:type() ~= "class_declaration" do
		node = node:parent()
	end
	return node
end

-- Extracts the variable under the cursor to the body of the strucut. Will add types to strings and positive integers
local extract_component = function()
	local call_expression = get_entire_call_expression()
	local enclosing_struct = get_enclosing_struct(call_expression)
	local range = { enclosing_struct:range() }
	local call_expression_changes = vim.split(vim.treesitter.get_node_text(call_expression, get_bufnr()), "\n")
	local new_struct_name = vim.fn.input("View name: ")
	local changes = { "", string.format("struct %s: View {", new_struct_name), "  var body: some View {" }

	for _, change in pairs(call_expression_changes) do
		table.insert(changes, change)
	end
	table.insert(changes, "   }")
	table.insert(changes, "}")
	vim.api.nvim_buf_set_lines(get_bufnr(), range[3] + 1, range[3] + 1, false, changes)
	local old_func_range = { call_expression:range() }
	local indentation = string.rep(" ", old_func_range[2])
	vim.api.nvim_buf_set_lines(
		get_bufnr(),
		old_func_range[1],
		old_func_range[3] + 1,
		false,
		{ indentation .. new_struct_name .. "()" }
	)
end

-- Extracts a variable to the struct's fields
local extract_variable_to_struct = function()
	local value_argument_node = get_current_node()
	while value_argument_node ~= nil and value_argument_node:field("value")[1] == nil do
		value_argument_node = value_argument_node:parent()
	end
	local value_node = value_argument_node:field("value")[1]
	local value_node_range = { value_node:range() }
	local var_type = ""
	local var_name = vim.fn.input("Variable name: ")

	if value_node:field("text")[1] ~= nil then
		var_type = "String"
	elseif value_node:type() == "integer_literal" or value_node:field("target")[1] == "integer_literal" then
		var_type = "Int"
	end

	vim.api.nvim_buf_set_text(
		get_bufnr(),
		value_node_range[1],
		value_node_range[2],
		value_node_range[3],
		value_node_range[4],
		{ var_name }
	)

	local class_declaration = get_enclosing_struct(value_node)
	local captures = property_declaration_query:iter_captures(class_declaration, get_bufnr(), 0, -1)
	local props = {}
	local is_not_body_field = function(node)
		return get_node_text(node) ~= "body"
	end
	for _, prop in captures do
		local name_field = prop:field("name")
		if name_field[1] ~= nil and is_not_body_field(name_field[1]) then
			table.insert(props, prop)
		end
	end
	local last_property_range = { props[#props]:range() }
	local line_after_last_property = last_property_range[3] + 1
	local var_declaration = string.format("  var %s: %s", var_name, var_type)
	vim.api.nvim_buf_set_lines(
		get_bufnr(),
		line_after_last_property,
		line_after_last_property,
		false,
		{ var_declaration }
	)
	vim.api.nvim_win_set_cursor(get_winr(), { line_after_last_property + 1, var_declaration:len() })
end

local actions = {
	extract_component = function()
		return extract_component
	end,
	extract_variable_to_struct = function()
		return extract_variable_to_struct
	end,
	add_modifier = add_modifier,
}
return actions
