use std::{path::PathBuf, process::Command};

use log::info;

pub fn get_changed_files(base: &str) -> Vec<PathBuf> {
    let diff = Command::new("git")
        .args(["diff", "--name-only", &format!("{base}...HEAD")])
        .output()
        .expect("git diff failed");

    let mut files: Vec<PathBuf> = String::from_utf8_lossy(&diff.stdout)
        .lines()
        .map(PathBuf::from)
        .collect();

    let status = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .expect("git status failed");

    for line in String::from_utf8_lossy(&status.stdout).lines() {
        if let Some(path) = line.get(3..) {
            files.push(PathBuf::from(path));
        }
    }

    info!(
        "{} files changed in diff {}...HEAD incuding staged/unstaged",
        files.len(),
        base
    );

    files.sort();
    files.dedup();

    files
}
pub fn is_git_repo() -> bool {
    let output = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()
        .expect("not a git project");

    output.stderr.is_empty()
}
