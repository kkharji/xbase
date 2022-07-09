use std::{ffi::OsStr, fmt::Display, string::String};

/// Kill process using kill command
pub async fn kill_process_by_pid(pid_str: &String) -> anyhow::Result<bool> {
    Ok(tokio::process::Command::new("kill")
        .arg("-15")
        .arg(pid_str)
        .output()
        .await?
        .status
        .success())
}

/// Check if a process exists with a given pid
#[allow(dead_code)]
pub fn is_valid_pid(pid: &i32, cb: impl FnOnce()) -> bool {
    if libproc::libproc::proc_pid::name(*pid).is_err() {
        cb();
        false
    } else {
        true
    }
}

/// Get process pid by name.
///
/// If an error occured during searching  an error will be returned,
/// otherwise process with given name is not, an  Error::Lookup will be returned.
///
/// WARNNING: The first match will be returned, and duplicates will be ignored
pub fn get_pid_by_name<S>(name: S) -> crate::Result<i32>
where
    S: AsRef<OsStr> + Display,
    String: PartialEq<S>,
{
    use libproc::libproc::proc_pid;

    let pids = proc_pid::listpids(proc_pid::ProcType::ProcAllPIDS)?;

    for pid in pids {
        let pid = pid as i32;
        match proc_pid::name(pid).ok() {
            Some(process) if process.eq(&name) => return Ok(pid),
            _ => continue,
        }
    }

    Err(crate::Error::Lookup("Process".into(), format!("{name}")))
}

#[test]
fn test_get_by_name() {
    let existing_process = get_pid_by_name("DockHelper");
    let not_process = get_pid_by_name("afsd8439f");

    assert!(existing_process.is_ok());
    assert!(not_process.is_err());
}

#[test]
#[ignore = "internal"]
fn test_get_os_processes() {
    use libproc::libproc::proc_pid::*;
    let pids = listpids(ProcType::ProcAllPIDS).unwrap();

    for pid in pids {
        if let Some(name) = name(pid as i32).ok() {
            println!("{name}")
        }
    }
}
