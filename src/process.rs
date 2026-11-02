use std::{env, path::PathBuf};

#[derive(Debug)]
pub struct Process {
    pid: u32,
    command: String,
    cwd: Option<PathBuf>,
}

pub fn get_processes() -> Vec<Process> {
    vec![]
}
