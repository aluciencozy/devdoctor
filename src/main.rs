use std::collections::HashSet;
use std::fs;
use std::path::Path;
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

fn get_version(path: &Path) -> Result<String, &str> {
    let output = Command::new(path).arg("--version").output();

    match output {
        Ok(out) => {
            if out.status.success() {
                Ok(String::from(String::from_utf8_lossy(&out.stdout).trim()))
            } else {
                Err("exited with non-zero code")
            }
        }
        // i can't figure out how to return the error to the caller so i will print, figure this out later
        Err(e) => {
            println!("process failed with error {e}");
            Err("process failed")
        }
    }
}

fn main() {
    let executables = ["python", "rustc", "cargo", "node"];

    for exec in executables {
        match find_executable(exec) {
            Some(exec_path) => {
                let version = get_version(&exec_path);
                match version {
                    Ok(v) => println!("{exec} -> {} -> {}", exec_path.display(), v),
                    Err(e) => println!("process failed with error {e}"),
                }
            }
            None => println!("{exec} not found"),
        }
    }
}
