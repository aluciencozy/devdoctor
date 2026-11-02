use sysinfo::{Pid, System};

use std::{ffi::OsString, path::PathBuf};

#[derive(Debug)]
pub struct Process {
    pid: Pid,
    command: Vec<OsString>,
    cwd: Option<PathBuf>,
}

pub fn get_processes(sys: &System) -> Vec<Process> {
    sys.processes()
        .values()
        .map(|p| Process {
            pid: p.pid(),
            command: p.cmd().to_owned(),
            cwd: p.cwd().map(|path| path.to_path_buf()),
        })
        .collect()
}
