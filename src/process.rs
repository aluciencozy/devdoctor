use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

use std::{ffi::OsString, path::PathBuf};

#[derive(Debug)]
pub struct Process {
    pub(crate) pid: Pid,
    command: Vec<OsString>,
    cwd: Option<PathBuf>,
}

pub fn get_processes(sys: &mut System) -> Vec<Process> {
    let r = ProcessRefreshKind::nothing();
    let r = r
        .with_cwd(UpdateKind::Always)
        .with_cmd(UpdateKind::OnlyIfNotSet);

    sys.refresh_processes_specifics(ProcessesToUpdate::All, true, r);

    sys.processes()
        .values()
        .map(|p| Process {
            pid: p.pid(),
            command: p.cmd().to_owned(),
            cwd: p.cwd().map(|path| path.to_path_buf()),
        })
        .collect()
}
