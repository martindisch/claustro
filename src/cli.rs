use anyhow::{Context, Result, anyhow};
use clap::Parser;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(
    name = "claustro",
    about = "Run Claude Code inside a Docker container with selected host directories mounted into its workspace."
)]
pub struct Cli {
    /// Path to a directory containing a Dockerfile. The image is tagged <basename>:latest.
    #[arg(long)]
    pub image: PathBuf,

    /// Host directories to expose under /workspace/<basename> in the container.
    #[arg(required = true, num_args = 1..)]
    pub mounts: Vec<PathBuf>,

    /// Extra arguments forwarded to `claude` inside the container (after `--`).
    #[arg(last = true)]
    pub claude_args: Vec<String>,
}

pub fn derive_image_tag(image_dir: &Path) -> Result<String> {
    if !image_dir.is_dir() {
        return Err(anyhow!(
            "--image path is not a directory: {}",
            image_dir.display()
        ));
    }
    if !image_dir.join("Dockerfile").is_file() {
        return Err(anyhow!("no Dockerfile found in {}", image_dir.display()));
    }
    let canonical = image_dir
        .canonicalize()
        .with_context(|| format!("canonicalizing {}", image_dir.display()))?;
    let name = canonical
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow!("cannot derive image name from {}", canonical.display()))?;
    Ok(format!("{name}:latest"))
}
