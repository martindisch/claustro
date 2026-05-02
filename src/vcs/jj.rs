use eyre::{Result, WrapErr, eyre};
use std::path::Path;
use std::process::Command;

pub fn existing_workspace_names(repo: &Path) -> Result<Vec<String>> {
    let output = Command::new(jj_binary())
        .arg("workspace")
        .arg("list")
        .current_dir(repo)
        .output()
        .wrap_err_with(|| format!("Running `jj workspace list` in {}", repo.display()))?;

    if !output.status.success() {
        return Err(eyre!(
            "`jj workspace list` failed in {}: {}",
            repo.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    let names = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| line.split(':').next())
        .map(|s| s.trim().to_string())
        .collect();
    Ok(names)
}

pub fn create_workspace(repo: &Path, target: &Path, name: &str, debug: bool) -> Result<()> {
    let mut cmd = Command::new(jj_binary());
    cmd.arg("workspace")
        .arg("add")
        .arg(target)
        .arg("--name")
        .arg(name)
        .current_dir(repo);

    let context = || format!("Running `jj workspace add` in {}", repo.display());
    if debug {
        let status = cmd.status().wrap_err_with(context)?;
        if !status.success() {
            return Err(eyre!(
                "`jj workspace add` failed in {} ({})",
                repo.display(),
                status
            ));
        }
    } else {
        let output = cmd.output().wrap_err_with(context)?;
        if !output.status.success() {
            return Err(eyre!(
                "`jj workspace add` failed in {}: {}",
                repo.display(),
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
    }
    Ok(())
}

pub fn cleanup_workspace(repo: &Path, target: &Path, name: &str) {
    // Trigger an auto-snapshot of any pending changes so they survive as
    // commits in the source repo before the workspace is forgotten.
    let _ = Command::new(jj_binary())
        .arg("status")
        .current_dir(target)
        .output();
    // Forget the workspace so the source repo's workspace list stays clean.
    let _ = Command::new(jj_binary())
        .arg("workspace")
        .arg("forget")
        .arg(name)
        .current_dir(repo)
        .output();
}

fn jj_binary() -> String {
    std::env::var("JJ_BINARY").unwrap_or_else(|_| "jj".to_string())
}
