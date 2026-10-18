use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::{env, path::PathBuf};

#[derive(Debug)]
struct Runtime {
    name: String,
    exec_path: Option<PathBuf>,
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
        let mut version = None;
        let exec_path = find_executable(exec);

        if let Some(e) = &exec_path {
            version = get_version(e).ok();
        }

        runtimes.push(Runtime {
            name: exec.to_string(),
            exec_path,
            version,
        })
    }

    runtimes
}

fn main() {
    let runtimes = get_runtimes();

    for runtime in &runtimes {
        println!("{:?}", runtime);
    }
}
