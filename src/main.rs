mod port;
mod process;
mod runtime;

use std::collections::HashMap;

use sysinfo::{Pid, System};

use port::{ListeningSocket, get_ports};
use process::get_processes;
use runtime::get_runtimes;

fn main() {
    let Ok(mut sys) = System::new() else {
        println!("failed to initialize system");
        return;
    };

    let runtimes = get_runtimes();

    let processes_by_pid: HashMap<_, _> = get_processes(&mut sys)
        .into_iter()
        .map(|proc| (proc.pid, proc))
        .collect();

    let listeners = get_ports().unwrap_or_default();
    let mut sockets_by_pid: HashMap<Pid, Vec<ListeningSocket>> = HashMap::new();

    for socket in listeners {
        sockets_by_pid.entry(socket.pid).or_default().push(socket);
    }
}
