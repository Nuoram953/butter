use std::{path::PathBuf, process::Command};

use log::{debug, info};

pub fn get_changed_files(base: &str) -> Vec<PathBuf> {
    let output = Command::new("git")
        .args(["diff", "--name-only", &format!("{base}...HEAD")])
        .output()
        .expect("git diff failed");

    let files: Vec<PathBuf> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(PathBuf::from)
        .collect();

    info!("{} files changed in diff {}...HEAD", files.len(), base);

    debug!("files changed {:?}", files);

    files
}

pub fn is_git_repo() -> bool {
    let output = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()
        .expect("not a git project");

    output.stderr.is_empty()
}
