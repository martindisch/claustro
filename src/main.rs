mod auth;
mod cli;
mod docker;
mod mounts;

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    let resolved_mounts = mounts::resolve(&cli.mounts)?;
    let image_tag = cli::derive_image_tag(&cli.image)?;

    ctrlc::set_handler(|| {}).ok();

    docker::build(&cli.image, &image_tag)?;

    let session = auth::prepare_session()?;

    let status = docker::run(
        &image_tag,
        &resolved_mounts,
        session.path(),
        &cli.claude_args,
    )?;

    drop(session);

    std::process::exit(status.code().unwrap_or(1));
}
