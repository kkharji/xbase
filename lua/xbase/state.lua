---@class XBaseDeviceLookup
---@field id string The id of the device
---@field name string The name of the device

local M = {
  --- Devices index by platform
  ---@type table<string, XBaseDeviceLookup>
  runners = nil,
  ---@type table<string, table>
  project_info = {},
}

local assert_devices = function(available_devices, devices, device_filter)
  if #devices == 0 then
    local filter = "config: " .. table.concat(device_filter, ",")
    local available = "available: "
      .. table.concat(
        vim.tbl_map(function(d)
          return d.name
        end, available_devices),
        ","
      )
    error(string.format("No runners available based on user config %s %s.", filter, available))
  end
end

---Get Devices based on platform
---@param platform string
---@return XBaseDeviceLookup[]?
M.devices = function(platform)
  local available_devices = M.runners[platform]
  local devices = available_devices
  local device_filter = require("xbase.config").values.simctl[platform] or {}

  if #device_filter ~= 0 then
    devices = vim.tbl_filter(function(mem)
      return vim.tbl_contains(device_filter, mem.name)
    end, available_devices)

    assert_devices(devices)
  end

  if #devices == 0 then
    return nil
  end

  return devices
end

---Check Whether the configuration settings is being watched
---@param settings XBaseSettings
---@param command XBaseCommand
---@param device XBaseDeviceLookup?
---@param root string
---@return boolean
function M.is_watching(settings, command, device, root)
  local key = ("%s:%s:"):format(root, command)

  if command == "Run" then
    key = key .. (device ~= nil and device.name or "Bin") .. ":"
  end

  key = key .. "-configuration " .. settings.configuration
  key = key .. " -target " .. settings.target

  return vim.tbl_contains(M.project_info.watchlist or {}, key)
end

return M
