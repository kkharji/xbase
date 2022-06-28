// use tokio::sync::{Mutex, MutexGuard};

// use mlua::prelude::*;
// use serde::{Deserialize, Serialize};
// use tokio::sync::OnceCell;
// use xbase_proto::{Error, Result};

// #[derive(Clone, Debug, Serialize, Deserialize)]
// pub struct NvimState {
//     pub buffer: NvimLogBuffer,
// }

// impl NvimState {
//     pub fn new(lua: &Lua) -> Result<Self> {
//         Ok(Self {
//             buffer: NvimLogBuffer::new(lua)?,
//         })
//     }
// }

// static STATE: OnceCell<Mutex<NvimState>> = OnceCell::const_new();

// pub async fn state(lua: &Lua) -> Result<MutexGuard<'static, NvimState>> {
//     let state = STATE
//         .get_or_try_init(|| async { Ok::<_, Error>(Mutex::new(NvimState::new(lua)?)) })
//         .await?;
//     Ok(state.lock().await)
// }

// /// Buffer state
// #[derive(Clone, Debug, Serialize, Deserialize)]
// pub struct NvimLogBuffer {
//     bufnr: usize,
//     last_task_status: LoggingTaskStatus,
//     connected_tasks: Vec<LoggingTask>,
//     line_count: usize,
// }

// impl NvimLogBuffer {
//     /// Setup Nvim Log buffer
//     pub fn new(lua: &Lua) -> Result<Self> {
//         let bufnr = 0;

//         Ok(Self {
//             bufnr,
//             last_task_status: Default::default(),
//             connected_tasks: Default::default(),
//             line_count: 0,
//         })
//     }
//     fn get_last_status(&self, lua: &Lua) -> Result<LoggingTaskStatus> {
//         todo!()
//     }

//     fn set_last_status(&mut self, lua: &Lua, success: LoggingTaskStatus) -> Result<()> {
//         todo!()
//     }

//     fn append<S: AsRef<str>>(&self, lua: &Lua, chunk: S) -> Result<()> {
//         todo!()
//     }

//     fn push<S: AsRef<str>>(&self, lua: &Lua, line: S) -> Result<()> {
//         todo!()
//     }

//     fn clear(&self, lua: &Lua) -> Result<()> {
//         todo!()
//     }

//     fn open(&self, lua: &Lua, direction: BufferDirection) -> Result<()> {
//         todo!()
//     }

//     fn close(&self, lua: &Lua) -> Result<()> {
//         todo!()
//     }

//     fn insert_task(&mut self, lua: &Lua, task: LoggingTask) -> Result<()> {
//         todo!()
//     }

//     fn tasks(&self) -> Vec<&LoggingTask> {
//         todo!()
//     }
// }
