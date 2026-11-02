mod process;
mod runtime;

use sysinfo::System;

use process::get_processes;
use runtime::get_runtimes;

fn main() {
    let Ok(mut sys) = System::new_all() else {
        println!("failed to initialize system");
        return;
    };

    sys.refresh_all();

    println!("total memory: {} bytes", sys.total_memory());
    println!("used memory : {} bytes", sys.used_memory());
    println!("total swap  : {} bytes", sys.total_swap());
    println!("used swap   : {} bytes", sys.used_swap());

    println!("NB CPUs: {}", sys.cpus().len());
    let runtimes = get_runtimes();

    for runtime in &runtimes {
        println!("{runtime:?}");
    }

    let processes = get_processes(&sys);

    for process in &processes {
        println!("{process:?}");
    }
}
