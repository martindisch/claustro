use crate::auth::SessionDirectory;
use crate::mounts::{ResolvedMount, to_docker_source};
use eyre::{Result, WrapErr, eyre};
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
        .wrap_err("Invoking `docker build` (is Docker installed and on PATH?)")?;

    if !status.success() {
        return Err(eyre!("Docker build failed with {status}"));
    }

    Ok(())
}

pub fn run(
    image_tag: &str,
    mounts: &[ResolvedMount],
    session: &SessionDirectory,
    claude_args: &[String],
) -> Result<ExitStatus> {
    let mut cmd = Command::new("docker");
    cmd.arg("run").arg("--rm").arg("-it");

    let claude_dir_src = to_docker_source(&session.claude_dir);
    cmd.arg("--mount").arg(format!(
        "type=bind,source={claude_dir_src},target=/root/.claude",
    ));

    if let Some(user_config) = &session.user_config {
        let user_config_src = to_docker_source(user_config);
        cmd.arg("--mount").arg(format!(
            "type=bind,source={user_config_src},target=/root/.claude.json",
        ));
    }

    for mount in mounts {
        let src = to_docker_source(&mount.host_path);
        cmd.arg("--mount").arg(format!(
            "type=bind,source={src},target=/workspace/{directory_name}",
            directory_name = mount.directory_name
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

    cmd.status().wrap_err("Invoking `docker run`")
}
