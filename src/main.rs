mod auth;
mod cli;
mod docker;
mod mounts;
mod terminal;
mod vcs;
mod workspaces;

use clap::Parser;
use eyre::Result;
use std::process::ExitCode;

fn main() -> Result<ExitCode> {
    let cli = cli::Cli::parse();

    let resolved_mounts = mounts::resolve(&cli.mounts)?;
    let image_tag = cli::derive_image_tag(&cli.image)?;

    docker::build(&cli.image, &image_tag, cli.debug)?;

    // The host's GitHub Copilot token, forwarded into the container so it can
    // authenticate without persisting any credentials on disk.
    let copilot_token = auth::read_copilot_token()?;

    // Per-repo jj workspaces in a temp dir; the dir is mounted at /workspace.
    // Workspaces snapshot pending changes and are forgotten on cleanup.
    let workspaces = workspaces::create(&resolved_mounts, cli.debug)?;

    terminal::open_workspace_tabs(&workspaces)?;

    let status = docker::run(
        &image_tag,
        workspaces.path(),
        &copilot_token,
        &cli.copilot_args,
    )?;

    Ok(ExitCode::from(status.code().unwrap_or(1) as u8))
}
