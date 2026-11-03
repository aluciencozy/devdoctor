mod port;
mod process;
mod runtime;

use std::collections::HashMap;

use sysinfo::{Pid, System};

use port::{Port, get_ports};
use process::get_processes;
use runtime::get_runtimes;

fn main() {
    let Ok(mut sys) = System::new() else {
        println!("failed to initialize system");
        return;
    };

    let runtimes = get_runtimes();
    for runtime in &runtimes {
        println!("{runtime:?}");
    }

    let processes = get_processes(&mut sys);
    for process in &processes {
        println!("{process:?}");
    }

    let processes: HashMap<_, _> = processes.into_iter().map(|proc| (proc.pid, proc)).collect();

    let ports = get_ports();
    match ports {
        Ok(ports) => {
            for port in &ports {
                println!("{port:?}");
            }

            // match ports to pids, for now this is just handled in main
            let mut pid_to_ports: HashMap<Pid, Vec<Port>> = HashMap::new();
            for port in ports {
                if pid_to_ports.contains_key(&port.pid) {
                    if let Some(v) = pid_to_ports.get_mut(&port.pid) {
                        v.push(port);
                    }
                } else {
                    pid_to_ports.insert(port.pid, vec![port]);
                }
            }
        }
        Err(e) => println!("{e}"),
    }
}
