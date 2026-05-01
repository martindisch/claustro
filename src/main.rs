mod auth;
mod cli;
mod docker;
mod mounts;
mod workspaces;

use clap::Parser;
use eyre::Result;
use std::process::ExitCode;

fn main() -> Result<ExitCode> {
    let cli = cli::Cli::parse();

    let resolved_mounts = mounts::resolve(&cli.mounts)?;
    let image_tag = cli::derive_image_tag(&cli.image)?;

    docker::build(&cli.image, &image_tag, cli.debug)?;

    // Mounted temporary directory with a copy of the host credentials, so that
    // the container can authenticate but not affect the host credentials on disk.
    let session_directory = auth::prepare_session_directory()?;

    // Per-repo jj workspaces in a temp dir; the dir is mounted at /workspace.
    // Workspaces snapshot pending changes and are forgotten on cleanup.
    let workspaces = workspaces::create(&resolved_mounts, cli.debug)?;

    let status = docker::run(
        &image_tag,
        workspaces.path(),
        &session_directory,
        &cli.claude_args,
    )?;

    Ok(ExitCode::from(status.code().unwrap_or(1) as u8))
}
