local lib = require "libxcodebase"
local M = {}

M.start = function(opts)
  lib.watch_start(opts)
end

M.stop = function()
  lib.watch_stop {}
end

M.is_watching = false

return M
