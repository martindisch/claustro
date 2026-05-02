use crate::mounts::ResolvedMount;
use crate::vcs::Vcs;
use eyre::{Result, WrapErr};
use indicatif::ProgressBar;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tempfile::TempDir;

pub struct ContainerWorkspaces {
    temp: TempDir,
    workspaces: Vec<Workspace>,
    debug: bool,
}

struct Workspace {
    vcs: Vcs,
    repo: PathBuf,
    repo_name: String,
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
        let spinner = (!self.debug).then(|| {
            let pb = ProgressBar::new_spinner();
            pb.enable_steady_tick(Duration::from_millis(100));
            pb
        });

        for ws in &self.workspaces {
            if let Some(s) = &spinner {
                s.set_message(format!("Cleaning up workspace for {}", ws.repo_name));
            }
            ws.vcs.cleanup_workspace(&ws.repo, &ws.dir, &ws.name);
        }

        if let Some(s) = spinner {
            s.finish_and_clear();
        }
    }
}

pub fn create(repos: &[ResolvedMount], debug: bool) -> Result<ContainerWorkspaces> {
    let temp = tempfile::Builder::new()
        .prefix("claustro-workspaces-")
        .tempdir()
        .wrap_err("Creating workspaces temp directory")?;

    let mut workspaces = Vec::with_capacity(repos.len());

    let spinner = (!debug).then(|| {
        let pb = ProgressBar::new_spinner();
        pb.enable_steady_tick(Duration::from_millis(100));
        pb
    });

    for repo in repos {
        if let Some(s) = &spinner {
            s.set_message(format!("Preparing workspace for {}", repo.directory_name));
        }

        let vcs = Vcs::detect(&repo.host_path)?;
        let name = vcs.pick_workspace_name(&repo.host_path)?;
        let dir = temp.path().join(&repo.directory_name);
        vcs.create_workspace(&repo.host_path, &dir, &name, debug)?;

        workspaces.push(Workspace {
            vcs,
            repo: repo.host_path.clone(),
            repo_name: repo.directory_name.clone(),
            name,
            dir,
        });
    }

    if let Some(s) = spinner {
        s.finish_and_clear();
    }

    Ok(ContainerWorkspaces {
        temp,
        workspaces,
        debug,
    })
}
