mod process;
mod runtime;

use process::get_processes;
use runtime::get_runtimes;

fn main() {
    let runtimes = get_runtimes();

    for runtime in &runtimes {
        println!("{runtime:?}");
    }

    let processes = get_processes();

    for process in &processes {
        println!("{process:?}");
    }
}
