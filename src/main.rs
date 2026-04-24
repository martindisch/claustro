mod auth;
mod cli;
mod docker;
mod mounts;

use clap::Parser;
use eyre::Result;
use std::process::ExitCode;

fn main() -> Result<ExitCode> {
    let cli = cli::Cli::parse();

    let resolved_mounts = mounts::resolve(&cli.mounts)?;
    let image_tag = cli::derive_image_tag(&cli.image)?;

    // Best-effort to attempt to prevent Ctrl-C from killing the process before
    // we've cleaned up the temp dir
    ctrlc::set_handler(|| {})?;

    docker::build(&cli.image, &image_tag)?;

    // Mounted temporary directory with a copy of the host credentials, so that
    // the container can authenticate but not affect the host credentials on disk.
    let session_directory = auth::prepare_session_directory()?;

    let status = docker::run(
        &image_tag,
        &resolved_mounts,
        session_directory.path(),
        &cli.claude_args,
    )?;

    Ok(ExitCode::from(status.code().unwrap_or(1) as u8))
}
