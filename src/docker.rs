use crate::mounts::{ResolvedMount, to_docker_source};
use anyhow::{Context, Result, anyhow};
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};

pub fn build(context_dir: &Path, tag: &str) -> Result<()> {
    let status = Command::new("docker")
        .arg("build")
        .arg("-t")
        .arg(tag)
        .arg(context_dir)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("invoking `docker build` (is Docker installed and on PATH?)")?;

    if !status.success() {
        return Err(anyhow!("docker build failed with {status}"));
    }
    Ok(())
}

pub fn run(
    image_tag: &str,
    mounts: &[ResolvedMount],
    session_dir: &Path,
    claude_args: &[String],
) -> Result<ExitStatus> {
    let mut cmd = Command::new("docker");
    cmd.arg("run").arg("--rm").arg("-it");

    let session_src = to_docker_source(session_dir);
    cmd.arg("--mount").arg(format!(
        "type=bind,source={src},target=/root/.claude",
        src = session_src
    ));

    for m in mounts {
        let src = to_docker_source(&m.host_path);
        cmd.arg("--mount").arg(format!(
            "type=bind,source={src},target=/workspace/{name}",
            src = src,
            name = m.basename
        ));
    }

    cmd.arg("-w").arg("/workspace");
    cmd.arg(image_tag);
    cmd.arg("claude").arg("--dangerously-skip-permissions");
    for a in claude_args {
        cmd.arg(a);
    }

    cmd.stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    cmd.status().context("invoking `docker run`")
}
