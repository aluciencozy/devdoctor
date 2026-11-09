use process_control::{ChildExt, Control};
use std::path::Path;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

const VERSION_TIMEOUT: Duration = Duration::from_secs(3);

#[derive(Debug)]
pub(crate) struct Runtime {
    pub(crate) name: String,
    pub(crate) candidates: Result<Vec<ExecutableCandidate>, which::Error>,
}

#[derive(Debug)]
pub(crate) struct ExecutableCandidate {
    pub(crate) path: PathBuf,
    pub(crate) version: Result<String, VersionError>,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum VersionError {
    #[error("could not run version command: {0}")]
    Io(#[from] std::io::Error),

    #[error("version command exited with {status}: {stderr}")]
    Unsuccessful {
        status: std::process::ExitStatus,
        stderr: String,
    },

    #[error("version command succeeded but returned no vesrion text")]
    EmptyOutput,

    #[error("version command timed out after {limit:?}")]
    TimedOut { limit: Duration },
}

fn find_all_executables(executable: &str) -> Result<Vec<PathBuf>, which::Error> {
    let paths = which::which_all(executable)?;
    Ok(paths.collect())
}

fn run_version_command(
    command: &mut Command,
    timeout: Duration,
) -> Result<std::process::Output, VersionError> {
    let child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let output = child
        .controlled_with_output()
        .time_limit(timeout)
        .terminate_for_timeout()
        .wait()?
        .ok_or(VersionError::TimedOut { limit: timeout })?;

    Ok(output.into_std_lossy())
}

fn get_version(path: &Path) -> Result<String, VersionError> {
    let mut command = Command::new(path);
    command.arg("--version");

    let out = run_version_command(&mut command, VERSION_TIMEOUT)?;

    if !out.status.success() {
        return Err(VersionError::Unsuccessful {
            status: out.status,
            stderr: String::from_utf8_lossy(&out.stderr).trim().to_string(),
        });
    }

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);

    let version = if stdout.trim().is_empty() {
        stderr.trim()
    } else {
        stdout.trim()
    };

    if version.is_empty() {
        Err(VersionError::EmptyOutput)
    } else {
        Ok(version.to_owned())
    }
}

fn discover_candidates(executable: &str) -> Result<Vec<ExecutableCandidate>, which::Error> {
    let paths = find_all_executables(executable)?;

    let candidates = paths
        .into_iter()
        .map(|path| {
            let version = get_version(&path);
            ExecutableCandidate { path, version }
        })
        .collect();

    Ok(candidates)
}

pub(crate) fn get_runtimes() -> Vec<Runtime> {
    let executables = ["python", "rustc", "cargo", "node"];

    executables
        .into_iter()
        .map(|executable| Runtime {
            name: executable.to_string(),
            candidates: discover_candidates(executable),
        })
        .collect()
}
