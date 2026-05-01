use crate::auth::SessionDirectory;
use crate::mounts::to_docker_source;
use eyre::{Result, WrapErr, eyre};
use std::fs;
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};

const CLAUSTRO_LAYER_TEMPLATE: &str = include_str!("claustro_layer.dockerfile");
const CLAUSTRO_ENTRYPOINT: &str = include_str!("claustro_entrypoint");
const ZELLIJ_LAYOUT: &str = include_str!("zellij_layout.kdl");
const ZELLIJ_CONFIG: &str = include_str!("zellij_config.kdl");

pub fn build(image_dir: &Path, tag: &str, debug: bool) -> Result<()> {
    let inner_tag = inner_image_tag(tag);

    // Phase 1: build the user's image, which is expected to be a vanilla Dockerfile
    // that only installs whatever tools the user wants Claude to have access to.
    docker_build(image_dir, &inner_tag, debug)?;

    // Phase 2: wrap it with the claustro layer (entrypoint, claude user, workspace).
    let layer_dockerfile = CLAUSTRO_LAYER_TEMPLATE.replace("{INNER_IMAGE}", &inner_tag);
    let layer_context = tempfile::Builder::new()
        .prefix("claustro-layer-")
        .tempdir()
        .wrap_err("Creating layer build context")?;

    let dockerfile_path = layer_context.path().join("Dockerfile");
    fs::write(&dockerfile_path, layer_dockerfile)
        .wrap_err_with(|| format!("Writing {}", dockerfile_path.display()))?;

    let entrypoint_path = layer_context.path().join("claustro-entrypoint");
    fs::write(&entrypoint_path, CLAUSTRO_ENTRYPOINT)
        .wrap_err_with(|| format!("Writing {}", entrypoint_path.display()))?;

    let zellij_layout_path = layer_context.path().join("zellij_layout.kdl");
    fs::write(&zellij_layout_path, ZELLIJ_LAYOUT)
        .wrap_err_with(|| format!("Writing {}", zellij_layout_path.display()))?;

    let zellij_config_path = layer_context.path().join("zellij_config.kdl");
    fs::write(&zellij_config_path, ZELLIJ_CONFIG)
        .wrap_err_with(|| format!("Writing {}", zellij_config_path.display()))?;

    docker_build(layer_context.path(), tag, debug)?;

    Ok(())
}

fn docker_build(context_dir: &Path, tag: &str, debug: bool) -> Result<()> {
    let mut cmd = Command::new("docker");
    cmd.arg("build").arg("-t").arg(tag).arg(context_dir);

    if debug {
        let status = cmd
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .wrap_err("Invoking `docker build` (is Docker installed and on PATH?)")?;
        if !status.success() {
            return Err(eyre!("Docker build failed with {status}"));
        }
    } else {
        let output = cmd
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .output()
            .wrap_err("Invoking `docker build` (is Docker installed and on PATH?)")?;
        if !output.status.success() {
            return Err(eyre!(
                "Docker build failed:\n{}",
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
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
    workspaces_dir: &Path,
    session: &SessionDirectory,
    claude_args: &[String],
) -> Result<ExitStatus> {
    let mut cmd = Command::new("docker");
    cmd.arg("run").arg("--rm").arg("-it");

    let claude_dir_src = to_docker_source(&session.claude_dir);
    cmd.arg("--mount").arg(format!(
        "type=bind,source={claude_dir_src},target=/home/claude/.claude",
    ));

    let user_config_src = to_docker_source(&session.user_config);
    cmd.arg("--mount").arg(format!(
        "type=bind,source={user_config_src},target=/home/claude/.claude.json",
    ));

    let workspaces_src = to_docker_source(workspaces_dir);
    cmd.arg("--mount").arg(format!(
        "type=bind,source={workspaces_src},target=/workspace",
    ));

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
