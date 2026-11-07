use color_eyre::Result;
use snapshot::SystemSnapshot;
use sysinfo::System;

mod listener;
mod process;
mod runtime;
mod snapshot;

fn main() -> Result<()> {
    color_eyre::install()?;

    let mut sys = System::new()?;
    let mut snapshot = SystemSnapshot::new(&mut sys);

    // The terminal interface will go here.

    Ok(())
}
