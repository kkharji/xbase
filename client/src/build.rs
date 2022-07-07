use crate::broadcast::Broadcast;
use crate::runtime;

use std::path::PathBuf;
use tap::Pipe;
use xbase_proto::*;

// pub unsafe fn "C" xbase_build(req: BuildRequest) {
//     runtime::request!(build, req).ok();
// }
