use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::{env, path::PathBuf};

struct Runtime {
    name: String,
    exec_path: PathBuf,
    version: Option<String>,
}

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

fn get_version(path: &Path) -> Result<String, String> {
    let out = Command::new(path)
        .arg("--version")
        .output()
        .map_err(|e| e.to_string())?;

    if out.status.success() {
        Ok(String::from(String::from_utf8_lossy(&out.stdout).trim()))
    } else {
        let e = format!(
            "Process failed\n{}\nstderr:{}",
            out.status,
            String::from_utf8_lossy(&out.stderr)
        );
        Err(e)
    }
}

fn get_runtimes() -> Vec<Runtime> {
    let executables = ["python", "rustc", "cargo", "node"];
    let mut runtimes: Vec<Runtime> = Vec::new();

    for exec in executables {
        match find_executable(exec) {
            Some(exec_path) => {
                let version = get_version(&exec_path).ok();

                runtimes.push(Runtime {
                    name: exec.to_string(),
                    exec_path,
                    version,
                })
            }
            None => println!("{exec} not found"),
        }
    }

    runtimes
}

fn main() {
    let runtimes = get_runtimes();

    for runtime in &runtimes {
        let print_str = format!("{} -> {} ->", runtime.name, runtime.exec_path.display());
        match &runtime.version {
            Some(v) => {
                println!("{} {}", print_str, v);
            }
            None => {
                println!("{} version check failed", print_str);
            }
        }
    }
}
