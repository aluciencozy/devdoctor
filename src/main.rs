mod port;
mod process;
mod runtime;

use sysinfo::System;

use port::get_ports;
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

    let ports = get_ports();
    for port in &ports {
        println!("{port:?}");
    }
}
