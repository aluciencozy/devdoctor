use std::collections::HashSet;
use std::fs;
use std::process::Command;
use std::{env, path::PathBuf};

fn find_executable(executable: &str) -> Option<PathBuf> {
    let path_env = env::var_os("PATH").expect("path environment variable not found");

    let paths_split = env::split_paths(&path_env);

    for mut path in paths_split {
        path.push(executable);
        if path.exists() {
            return Some(path);
        }
    }

    None
}

fn find_all_executables(executable: &str) -> Vec<PathBuf> {
    let path_env = env::var_os("PATH").expect("path environment variable not found");

    let paths_split = env::split_paths(&path_env);

    let mut all_paths: Vec<PathBuf> = Vec::new();
    let mut seen_path: HashSet<PathBuf> = HashSet::new();

    for mut path in paths_split {
        path.push(executable);

        if !path.exists() {
            continue;
        }

        match fs::canonicalize(&path) {
            Ok(canonicalized_path) => {
                if seen_path.contains(&canonicalized_path) {
                    continue;
                }

                all_paths.push(path);
                seen_path.insert(canonicalized_path);
            }
            Err(e) => {
                println!(
                    "error canonicalizing path: {}\nskipping with error: {e}",
                    path.display()
                );
                continue;
            }
        };
    }

    all_paths
}

fn main() {
    let executables = ["python", "rustc", "cargo", "node"];

    for exec in executables {
        match find_executable(exec) {
            Some(exec_path) => {
                let version = Command::new(&exec_path)
                    .arg("--version")
                    .output()
                    .expect("failed to execute process");

                println!("status: {}", version.status);
                println!(
                    "stdout: {exec} -> {} -> {}",
                    exec_path.display(),
                    String::from_utf8_lossy(&version.stdout)
                );
                println!("stderr: {}", String::from_utf8_lossy(&version.stderr));
            }
            None => println!("{exec} not found"),
        }
    }
}
