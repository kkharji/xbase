use mlua::prelude::*;
use serde::{Deserialize, Serialize};
use xbase_proto::{BufferDirection, Result};
use xbase_proto::{LoggingTask, LoggingTaskStatus};

/// Buffer state
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NvimLogBuffer {
    bufnr: usize,
    last_task_status: LoggingTaskStatus,
    connected_tasks: Vec<LoggingTask>,
    line_count: usize,
}

impl NvimLogBuffer {
    /// Setup Nvim Log buffer
    pub fn new(lua: &Lua) -> Result<Self> {
        let bufnr = 0;

        Ok(Self {
            bufnr,
            last_task_status: Default::default(),
            connected_tasks: Default::default(),
            line_count: 0,
        })
    }
    fn get_last_status(&self, lua: &Lua) -> Result<LoggingTaskStatus> {
        todo!()
    }

    fn set_last_status(&mut self, lua: &Lua, success: LoggingTaskStatus) -> Result<()> {
        todo!()
    }

    fn append<S: AsRef<str>>(&self, lua: &Lua, chunk: S) -> Result<()> {
        todo!()
    }

    fn push<S: AsRef<str>>(&self, lua: &Lua, line: S) -> Result<()> {
        todo!()
    }

    fn clear(&self, lua: &Lua) -> Result<()> {
        todo!()
    }

    fn open(&self, lua: &Lua, direction: BufferDirection) -> Result<()> {
        todo!()
    }

    fn close(&self, lua: &Lua) -> Result<()> {
        todo!()
    }

    fn insert_task(&mut self, lua: &Lua, task: LoggingTask) -> Result<()> {
        todo!()
    }

    fn tasks(&self) -> Vec<&LoggingTask> {
        todo!()
    }
}

impl<'lua> FromLua<'lua> for NvimLogBuffer {
    fn from_lua(v: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
        if let LuaValue::Table(table) = v {
            Ok(Self {
                bufnr: table.get("bufnr")?,
                last_task_status: table.get("last_task_status")?,
                connected_tasks: table.get("connected_tasks")?,
                line_count: table.get("line_count")?,
            })
        } else {
            Err(LuaError::external("NvimLogBuffer is not initialized!"))
        }
    }
}

impl<'lua> ToLua<'lua> for NvimLogBuffer {
    fn to_lua(self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        let table = lua.create_table()?;
        table.set("bufnr", self.bufnr)?;
        table.set("last_task_status", self.last_task_status.to_string())?;
        table.set("line_count", self.line_count)?;
        let connected_tasks = lua.create_table()?;
        for (i, t) in self.connected_tasks.into_iter().enumerate() {
            connected_tasks.set(i, t)?;
        }

        table.set("connected_tasks", connected_tasks)?;

        Ok(LuaValue::Table(table))
    }
}
