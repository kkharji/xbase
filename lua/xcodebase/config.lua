local config = {}
local log = require "xcodebase.log"

_XCODEBASECONFIG = _XCODEBASECONFIG or {}

config.values = _XCODEBASECONFIG

---@class XcodeBaseOptions
local defaults = {
  --- Log level. Set to error to ignore everything
  --- { "trace", "debug", "info", "warn", "error" }
  log_level = "debug",
  --- Default log buffer direction.
  --- { "horizontal", "vertical", "float" }
  default_log_buffer_direction = "horizontal",
  --- Statusline provider configurations
  statusline = {
    running = { icon = "⚙", color = "#e0af68" },
    success = { icon = "", color = "#1abc9c" },
    failure = { icon = "", color = "#db4b4b" },
    show_progress = false,
  },
}

--- Enahnced version of builtin type function that inclued list type.
---@param val any
---@return string
local get_type = function(val)
  local typ = type(val)
  if val == "table" then
    return vim.tbl_islist(val) and "list" or "table"
  else
    return typ
  end
end

--- returns true if the key name should be skipped when doing type checking.
---@param key string
---@return boolean: true if it should be if key skipped
local should_skip_type_checking = function(key)
  for _, v in ipairs { "mappings", "blacklist", "fenced", "templates" } do
    for _, k in ipairs(vim.split(key, "%.")) do
      if k:find(v) then
        return true
      end
    end
  end
  return false
end

--- Checks defaults values types against modification values.
--- skips type checking if the key match an item in `skip_type_checking`.
---@param dv any: defaults values
---@param mv any: custom values or modifications .
---@param trace string
---@return string: type of the default value
local check_type = function(dv, mv, trace)
  local dtype = get_type(dv)
  local mtype = get_type(mv)
  local skip = should_skip_type_checking(trace)

  --- hmm I'm not sure about this.
  if dv == nil and not skip then
    return log.error(("Invalid configuration key: `%s`"):format(trace))
  elseif dtype ~= mtype and not skip then
    return log.error(("Unexpcted configuration value for `%s`, expected %s, got %s"):format(trace, dtype, mtype))
  end

  return dtype
end

--- Consumes configuration options and sets the values of keys.
--- supports nested keys and values
---@param startkey string: the parent key
---@param d table: default configuration key
---@param m table: the value of startkey
local consume_opts
consume_opts = function(startkey, d, m)
  for k, v in pairs(m) do
    local typ = check_type(d[k], v, ("%s.%s"):format(startkey, k))
    if typ == "table" then
      consume_opts(startkey .. "." .. k, d[k], v)
    else
      d[k] = v
    end
  end
end

--- Set or extend defaults configuration
---@param opts table
config.set = function(opts)
  opts = opts or {}

  if next(opts) ~= nil then
    for k, v in pairs(opts) do
      local typ = check_type(_XCODEBASECONFIG[k], v, k)
      if typ ~= "table" then
        _XCODEBASECONFIG[k] = v
      else
        consume_opts(k, _XCODEBASECONFIG[k], v)
      end
    end
  else
    if vim.tbl_isempty(_XCODEBASECONFIG) then
      _XCODEBASECONFIG = defaults
      config.values = _XCODEBASECONFIG
    end
  end

  _XCODEBASELOG.level = opts.log_level or defaults.log_level
end

config.set()

return config
