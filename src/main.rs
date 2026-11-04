mod listener;
mod process;
mod runtime;
mod snapshot;

use snapshot::SystemSnapshot;
use sysinfo::System;

fn main() {
    let Ok(mut sys) = System::new() else {
        println!("failed to initialize system");
        return;
    };

    let mut system_snapshot = SystemSnapshot::new(&mut sys);

    println!("{system_snapshot:?}");

    system_snapshot.refresh(&mut sys);

    println!("{system_snapshot:?}");
}
