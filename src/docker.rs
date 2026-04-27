use crate::auth::SessionDirectory;
use crate::mounts::{ResolvedMount, to_docker_source};
use eyre::{Result, WrapErr, eyre};
use std::fs;
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};

const CLAUSTRO_LAYER_TEMPLATE: &str = include_str!("claustro_layer.dockerfile");

pub fn build(image_dir: &Path, tag: &str) -> Result<()> {
    let inner_tag = inner_image_tag(tag);

    // Phase 1: build the user's image, which is expected to be a vanilla Dockerfile
    // that only installs whatever tools the user wants Claude to have access to.
    docker_build(image_dir, &inner_tag)?;

    // Phase 2: wrap it with the claustro layer (entrypoint, claude user, workspace).
    let layer_dockerfile = CLAUSTRO_LAYER_TEMPLATE.replace("{INNER_IMAGE}", &inner_tag);
    let layer_context = tempfile::Builder::new()
        .prefix("claustro-layer-")
        .tempdir()
        .wrap_err("Creating layer build context")?;
    let dockerfile_path = layer_context.path().join("Dockerfile");
    fs::write(&dockerfile_path, layer_dockerfile)
        .wrap_err_with(|| format!("Writing {}", dockerfile_path.display()))?;

    docker_build(layer_context.path(), tag)?;

    Ok(())
}

fn docker_build(context_dir: &Path, tag: &str) -> Result<()> {
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

fn inner_image_tag(tag: &str) -> String {
    match tag.split_once(':') {
        Some((name, version)) => format!("{name}-base:{version}"),
        None => format!("{tag}-base"),
    }
}

pub fn run(
    image_tag: &str,
    mounts: &[ResolvedMount],
    session: &SessionDirectory,
    drop_to_bash: bool,
    claude_args: &[String],
) -> Result<ExitStatus> {
    let mut cmd = Command::new("docker");
    cmd.arg("run").arg("--rm").arg("-it");

    if drop_to_bash {
        cmd.arg("-e").arg("CLAUSTRO_DROP_TO_BASH=1");
    }

    let claude_dir_src = to_docker_source(&session.claude_dir);
    cmd.arg("--mount").arg(format!(
        "type=bind,source={claude_dir_src},target=/home/claude/.claude",
    ));

    let user_config_src = to_docker_source(&session.user_config);
    cmd.arg("--mount").arg(format!(
        "type=bind,source={user_config_src},target=/home/claude/.claude.json",
    ));

    for mount in mounts {
        let src = to_docker_source(&mount.host_path);
        cmd.arg("--mount").arg(format!(
            "type=bind,source={src},target=/workspace/{directory_name}",
            directory_name = mount.directory_name
        ));
    }

    cmd.arg("-w").arg("/workspace");
    cmd.arg(image_tag);
    cmd.arg("--dangerously-skip-permissions");
    for a in claude_args {
        cmd.arg(a);
    }

    cmd.stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    cmd.status().wrap_err("Invoking `docker run`")
}
