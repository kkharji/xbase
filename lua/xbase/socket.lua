local uv = require "luv"

---@class XBaseSocket @Object to communcate with xbase sockets
---@field _socket any
---@field _stream_error any
local M = {}
M.__index = M

function M:connect(address)
  local socket = uv.new_pipe(false)
  local self = setmetatable({ _socket = socket, _stream_error = nil }, M)
  socket:connect(address, function(err)
    self._stream_error = self._stream_error or err
  end)
  return self
end

function M:write(data)
  if self._stream_error then
    error(self._stream_error)
  end
  uv.write(self._socket, vim.json.encode(data), function(err)
    if err then
      print(self._stream_error or err)
    end
  end)
end

function M:read_start(cb)
  if self._stream_error then
    error(self._stream_error)
  end
  self._socket:read_start(function(err, chunk)
    if err then
      error(err)
    elseif chunk ~= nil then
      vim.schedule(function()
        cb(chunk)
      end)
    end
  end)
end

function M:read_stop()
  if self._stream_error then
    error(self._stream_error)
  end
  uv.read_stop(self._socket)
end

function M:close()
  uv.close(self._socket)
end

return M
