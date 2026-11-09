use std::collections::HashMap;

use sysinfo::{Pid, System};

use crate::listener::{ListeningSocket, get_listening_sockets};
use crate::process::{ProcessInfo, get_processes};
use crate::runtime::{Runtime, get_runtimes};

#[derive(Debug)]
pub(crate) struct SystemSnapshot {
    pub(crate) runtimes: Vec<Runtime>,
    pub(crate) processes_by_pid: HashMap<Pid, ProcessInfo>,
    pub(crate) sockets_by_pid: HashMap<Pid, Vec<ListeningSocket>>,
}

impl SystemSnapshot {
    pub(crate) fn new(sys: &mut System) -> Self {
        let runtimes = get_runtimes();

        let processes_by_pid: HashMap<_, _> = get_processes(sys)
            .into_iter()
            .map(|proc| (proc.pid, proc))
            .collect();

        let listeners = get_listening_sockets().unwrap_or_default();
        let mut sockets_by_pid: HashMap<Pid, Vec<ListeningSocket>> = HashMap::new();

        for socket in listeners {
            sockets_by_pid.entry(socket.pid).or_default().push(socket);
        }

        SystemSnapshot {
            runtimes,
            processes_by_pid,
            sockets_by_pid,
        }
    }

    pub(crate) fn refresh(&mut self, sys: &mut System) {
        self.processes_by_pid = get_processes(sys)
            .into_iter()
            .map(|proc| (proc.pid, proc))
            .collect();

        let mut sockets_by_pid: HashMap<Pid, Vec<ListeningSocket>> = HashMap::new();

        let listeners = get_listening_sockets().unwrap_or_default();
        for socket in listeners {
            sockets_by_pid.entry(socket.pid).or_default().push(socket);
        }

        self.sockets_by_pid = sockets_by_pid;

        self.runtimes = get_runtimes();
    }
}
