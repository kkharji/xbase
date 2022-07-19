local M = {}

M.TaskKind = {
  is_build = function(kind)
    return kind == "Build"
  end,
  is_generate = function(kind)
    return kind == "Generate"
  end,
  is_compile = function(kind)
    return kind == "Compile"
  end,
  is_run = function(kind)
    return kind == "Run"
  end,
  prefix = function(self, kind)
    if self.is_compile(kind) then
      return "Compiling", "Compiled"
    elseif self.is_generate(kind) then
      return "Generating", "Generated"
    elseif self.is_build(kind) then
      return "Building", "Built"
    elseif self.is_run(kind) then
      return "Running", "Running"
    end
  end,
}

M.TaskStatus = {
  is_failed = function(status)
    return status == "Failed"
  end,
  is_succeeded = function(status)
    return status == "Succeeded"
  end,
  is_processing = function(status)
    return status == "Processing"
  end,
}

M.Message = {
  is_notify = function(ty)
    return ty == "Notify"
  end,
  is_open_logger = function(ty)
    return ty == "OpenLogger"
  end,
  is_log = function(ty)
    return ty == "Log"
  end,
  is_reload_lsp_server = function(ty)
    return ty == "ReloadLspServer"
  end,
  task_is_update_current = function(ty)
    return ty == "UpdateCurrentTask"
  end,
  task_is_set_current = function(ty)
    return ty == "SetCurrentTask"
  end,
  task_is_finish_current = function(ty)
    return ty == "FinishCurrentTask"
  end,
  is_set_watching = function(ty)
    return ty == "SetWatching"
  end,
  is_set_state = function(ty)
    return ty == "SetState"
  end,
}

return M
