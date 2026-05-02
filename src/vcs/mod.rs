mod git;
mod jj;

use eyre::{Result, eyre};
use std::path::Path;

pub enum Vcs {
    Jj,
    Git,
}

impl Vcs {
    pub fn detect(repo: &Path) -> Result<Self> {
        // Prefer jj when both are present (colocated repos).
        if repo.join(".jj").exists() {
            Ok(Vcs::Jj)
        } else if repo.join(".git").exists() {
            Ok(Vcs::Git)
        } else {
            Err(eyre!("{} is not a jj or git repository", repo.display()))
        }
    }

    pub fn pick_workspace_name(&self, repo: &Path) -> Result<String> {
        let existing = match self {
            Vcs::Jj => jj::existing_workspace_names(repo)?,
            Vcs::Git => git::existing_workspace_names(repo)?,
        };
        Ok(next_workspace_name(&existing))
    }

    pub fn create_workspace(
        &self,
        repo: &Path,
        target: &Path,
        name: &str,
        debug: bool,
    ) -> Result<()> {
        match self {
            Vcs::Jj => jj::create_workspace(repo, target, name, debug),
            Vcs::Git => git::create_workspace(repo, target, name, debug),
        }
    }

    pub fn cleanup_workspace(&self, repo: &Path, target: &Path, name: &str) {
        match self {
            Vcs::Jj => jj::cleanup_workspace(repo, target, name),
            Vcs::Git => git::cleanup_workspace(repo, target, name),
        }
    }
}

fn next_workspace_name(existing: &[String]) -> String {
    let max_n = existing
        .iter()
        .filter_map(|name| {
            name.strip_prefix('c')
                .and_then(|rest| rest.parse::<u32>().ok())
        })
        .max();
    let next = max_n.map_or(1, |n| n + 1);
    format!("c{next}")
}
