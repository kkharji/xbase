use crate::BroadcastHandler;
use mlua::{chunk, prelude::*};
use tap::Pipe;
use xbase_proto::*;

use super::{NvimLog, NvimNotify};

impl BroadcastHandler for Lua {
    type Result = LuaResult<()>;

    fn handle(&self, msg: Message) -> Self::Result {
        match msg {
            Message::Notify { msg, level, .. } => self.notify(msg, level),
            Message::Log { msg, level, .. } => self.log(msg, level),
            Message::Execute(task) => match task {
                Task::UpdateStatusline(state) => state.to_string().pipe(|s| {
                    self.load(chunk!(vim.g.xbase_watch_build_status = $s))
                        .exec()
                }),
            },
        }
    }
}
