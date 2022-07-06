use crate::runtime::{roots, rt};
use os_pipe::PipeWriter;
use std::os::unix::io::IntoRawFd;
use std::thread::JoinHandle;
use std::{io::Write, path::PathBuf};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    net::UnixStream,
};
use xbase_proto::*;

pub struct Broadcast;

impl Broadcast {
    /// Initialize Broadcast message handler and return raw file descriptor
    pub fn init_or_skip(address: PathBuf, root: PathBuf) -> Result<i32> {
        let mut roots = roots().lock().unwrap();
        if !roots.contains_key(&root) {
            let (reader, writer) = os_pipe::pipe()?;
            let handle = Self::start_writer(writer, address);
            let fd = reader.into_raw_fd();

            roots.insert(root.clone(), (fd, handle));
            Ok(fd)
        } else {
            let (fd, _) = roots.get(&root).unwrap();
            // NOTE: Should we return different status?
            Ok(*fd)
        }
    }

    /// Start Broadcast reader with a writer and Broadcast socket address
    fn start_writer(mut writer: PipeWriter, address: PathBuf) -> JoinHandle<Result<()>> {
        std::thread::spawn(move || {
            rt().block_on(async move {
                let mut stream = UnixStream::connect(address).await?;

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
