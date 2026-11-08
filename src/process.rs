use std::{ffi::OsString, path::PathBuf};

use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessInfo {
    pub(crate) pid: Pid,
    pub(crate) command: Vec<OsString>,
    pub(crate) cwd: Option<PathBuf>,
}

pub fn get_processes(sys: &mut System) -> Vec<ProcessInfo> {
    let r = ProcessRefreshKind::nothing();
    let r = r
        .with_cwd(UpdateKind::Always)
        .with_cmd(UpdateKind::OnlyIfNotSet);

    sys.refresh_processes_specifics(ProcessesToUpdate::All, true, r);

    sys.processes()
        .values()
        .map(|p| ProcessInfo {
            pid: p.pid(),
            command: p.cmd().to_owned(),
            cwd: p.cwd().map(|path| path.to_path_buf()),
        })
        .collect()
}
