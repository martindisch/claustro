use crate::mounts::ResolvedMount;
use eyre::{Result, WrapErr, eyre};
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

pub struct ContainerWorkspaces {
    temp: TempDir,
    workspaces: Vec<Workspace>,
}

struct Workspace {
    repo: PathBuf,
    name: String,
    dir: PathBuf,
}

impl ContainerWorkspaces {
    pub fn path(&self) -> &Path {
        self.temp.path()
    }
}

impl Drop for ContainerWorkspaces {
    fn drop(&mut self) {
        for ws in &self.workspaces {
            // Trigger an auto-snapshot of any pending changes Claude made
            // inside the workspace, so they survive as commits in the source
            // repo before the workspace is forgotten.
            let _ = Command::new(jj_binary())
                .arg("status")
                .current_dir(&ws.dir)
                .output();
            // Forget the workspace so the source repo's workspace list stays
            // clean across claustro sessions.
            let _ = Command::new(jj_binary())
                .arg("workspace")
                .arg("forget")
                .arg(&ws.name)
                .current_dir(&ws.repo)
                .output();
        }
    }
}

pub fn create(repos: &[ResolvedMount], debug: bool) -> Result<ContainerWorkspaces> {
    let temp = tempfile::Builder::new()
        .prefix("claustro-workspaces-")
        .tempdir()
        .wrap_err("Creating workspaces temp directory")?;

    let mut workspaces = Vec::with_capacity(repos.len());

    for repo in repos {
        let name = pick_workspace_name(&repo.host_path)?;
        let dir = temp.path().join(&repo.directory_name);

        let mut cmd = Command::new(jj_binary());
        cmd.arg("workspace")
            .arg("add")
            .arg(&dir)
            .arg("--name")
            .arg(&name)
            .current_dir(&repo.host_path);

        let context = || format!("Running `jj workspace add` in {}", repo.host_path.display());
        if debug {
            let status = cmd.status().wrap_err_with(context)?;
            if !status.success() {
                return Err(eyre!(
                    "`jj workspace add` failed in {} ({})",
                    repo.host_path.display(),
                    status
                ));
            }
        } else {
            let output = cmd.output().wrap_err_with(context)?;
            if !output.status.success() {
                return Err(eyre!(
                    "`jj workspace add` failed in {}: {}",
                    repo.host_path.display(),
                    String::from_utf8_lossy(&output.stderr).trim()
                ));
            }
        }

        workspaces.push(Workspace {
            repo: repo.host_path.clone(),
            name,
            dir,
        });
    }

    Ok(ContainerWorkspaces { temp, workspaces })
}

fn pick_workspace_name(repo: &Path) -> Result<String> {
    let output = Command::new(jj_binary())
        .arg("workspace")
        .arg("list")
        .current_dir(repo)
        .output()
        .wrap_err_with(|| format!("Running `jj workspace list` in {}", repo.display()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(eyre!(
            "`jj workspace list` failed in {}: {}",
            repo.display(),
            stderr.trim()
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let max_n = stdout
        .lines()
        .filter_map(|line| line.split(':').next())
        .map(str::trim)
        .filter_map(|name| {
            name.strip_prefix('c')
                .and_then(|rest| rest.parse::<u32>().ok())
        })
        .max();

    let next = max_n.map_or(1, |n| n + 1);
    Ok(format!("c{next}"))
}

fn jj_binary() -> String {
    std::env::var("JJ_BINARY").unwrap_or_else(|_| "jj".to_string())
}
