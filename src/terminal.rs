use crate::workspaces::ContainerWorkspaces;
use eyre::{Result, WrapErr};
use std::process::{Command, Stdio};

/// Open a Windows Terminal tab for each mounted repo, with its cwd set to the
/// per-repo workspace under the temp directory.
pub fn open_workspace_tabs(workspaces: &ContainerWorkspaces) -> Result<()> {
    for workspace_dir in workspaces.dirs() {
        Command::new("wt.exe")
            .arg("-w")
            .arg("0")
            .arg("nt")
            .arg("-d")
            .arg(workspace_dir)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .wrap_err("Invoking `wt.exe` (is Windows Terminal installed and on PATH?)")?;
    }

    Ok(())
}
