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

    let session = auth::prepare_session()?;

    let status = docker::run(
        &image_tag,
        &resolved_mounts,
        session.path(),
        &cli.claude_args,
    )?;

    Ok(ExitCode::from(status.code().unwrap_or(1) as u8))
}
