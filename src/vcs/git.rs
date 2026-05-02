use eyre::{Result, WrapErr, eyre};
use std::path::Path;
use std::process::Command;

pub fn existing_workspace_names(repo: &Path) -> Result<Vec<String>> {
    // Branches outlive worktrees; scan branch names rather than active worktrees
    // so we don't reuse a name that's already taken.
    let output = Command::new(git_binary())
        .arg("branch")
        .arg("--list")
        .arg("c*")
        .arg("--format=%(refname:short)")
        .current_dir(repo)
        .output()
        .wrap_err_with(|| format!("Running `git branch --list` in {}", repo.display()))?;

    if !output.status.success() {
        return Err(eyre!(
            "`git branch --list` failed in {}: {}",
            repo.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    let names = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    Ok(names)
}

pub fn create_workspace(repo: &Path, target: &Path, name: &str, debug: bool) -> Result<()> {
    let mut cmd = Command::new(git_binary());
    cmd.arg("worktree")
        .arg("add")
        .arg("-b")
        .arg(name)
        .arg(target)
        .current_dir(repo);

    let context = || format!("Running `git worktree add` in {}", repo.display());
    if debug {
        let status = cmd.status().wrap_err_with(context)?;
        if !status.success() {
            return Err(eyre!(
                "`git worktree add` failed in {} ({})",
                repo.display(),
                status
            ));
        }
    } else {
        let output = cmd.output().wrap_err_with(context)?;
        if !output.status.success() {
            return Err(eyre!(
                "`git worktree add` failed in {}: {}",
                repo.display(),
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
    }
    Ok(())
}

pub fn cleanup_workspace(repo: &Path, target: &Path, name: &str) {
    // Stash any pending changes (tracked + untracked) into the source repo
    // so they survive worktree removal. Recoverable via `git stash list`.
    let _ = Command::new(git_binary())
        .arg("stash")
        .arg("push")
        .arg("--include-untracked")
        .arg("-m")
        .arg(format!("claustro {name}"))
        .current_dir(target)
        .output();
    // --force in case stash push had nothing to do but the worktree still has
    // index entries or other state git considers "dirty".
    let _ = Command::new(git_binary())
        .arg("worktree")
        .arg("remove")
        .arg("--force")
        .arg(target)
        .current_dir(repo)
        .output();
}

fn git_binary() -> String {
    std::env::var("GIT_BINARY").unwrap_or_else(|_| "git".to_string())
}
