use super::global::NvimNotify;
use crate::runtime::{rpc, rt};
use crate::BrodcastMessage;
use mlua::{chunk, prelude::*};
use once_cell::sync::Lazy;
use os_pipe::{PipeReader, PipeWriter};
use std::os::unix::io::IntoRawFd;
use std::sync::Mutex;
use std::thread::JoinHandle;
use std::{collections::HashMap, io::Write, path::PathBuf};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    net::UnixStream,
};
use xbase_proto::*;

static LISTENERS: Lazy<Mutex<HashMap<PathBuf, JoinHandle<Result<()>>>>> =
    Lazy::new(Default::default);

pub struct Listener;

impl Listener {
    /// Register a project and initialize command listener if the project isn't already initialized
    pub fn init_or_skip(lua: &Lua, root: PathBuf) -> LuaResult<()> {
        let mut listeners = LISTENERS.lock().unwrap();
        if !listeners.contains_key(&root) {
            let (reader, writer) = os_pipe::pipe()?;

            Listener::start_reader(lua, reader)?;
            let writer = Listener::start_writer(writer, root.clone());
            listeners.insert(root, writer);
        }
        Ok(())
    }

    /// Main handler of daemon messages
    fn handle(lua: &Lua, line: LuaString) -> LuaResult<()> {
        match lua.parse(line.as_bytes()) {
            Ok(msg) => lua.handle(msg),
            Err(err) => {
                lua.error(err.to_string()).ok();
                Ok(())
            }
        }
    }

    /// Setup and load a uv pipe to call [`Self::handle`] with read bytes
    pub fn start_reader(lua: &Lua, reader: PipeReader) -> LuaResult<()> {
        let reader_fd = reader.into_raw_fd();
        let callback = lua.create_function(Self::handle)?;

        // TODO: should closing be handled?
        lua.load(chunk! {
            local pipe = vim.loop.new_pipe()
            pipe:open($reader_fd)
            pipe:read_start(function(err, chunk)
                assert(not err, err)
                if chunk then
                    vim.schedule(function()
                         $callback(chunk)
                     end)
                end
            end)
        })
        .exec()
    }

    pub fn start_writer(mut writer: PipeWriter, root: PathBuf) -> JoinHandle<Result<()>> {
        std::thread::spawn(move || {
            rt().block_on(async move {
                let rpc = rpc().await;
                let client = Client {
                    pid: std::process::id() as i32,
                    root,
                };

                let address = rpc.register(context::current(), client).await??;
                let mut stream = UnixStream::connect(address).await?;
                drop(rpc);

                let (reader, _) = stream.split();
                let mut breader = BufReader::new(reader);
                let mut line = vec![];

                while let Ok(len) = breader.read_until(b'\n', &mut line).await {
                    if len == 0 {
                        break;
                    }

                    writer.write_all(line.as_slice()).ok();

                    line.clear();
                }

                OK(())
            })?;

            OK(())
        })
    }
}
