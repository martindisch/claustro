# Claustro

A thin wrapper that runs GitHub Copilot CLI inside a Docker container with
`--allow-all`, so Copilot has full agency without risking the
host system too much.

> [!NOTE]
> This is a simple tool and very specific to my setup, so probably not for you.

> [!CAUTION]
> While everything runs in a Docker container, this probably doesn't hold up
> against targeted exploitation.

## What it does

- Builds a user-supplied Dockerfile that provides the desired toolchain and
  layers a small entrypoint on top
- Forwards the host's `COPILOT_GITHUB_TOKEN` into the container as the same
  environment variable, so Copilot can authenticate without any credentials
  being written to disk
- For each repository you pass, creates a fresh jj workspace or git worktree in
  a temp directory and mounts that at `/workspace/<repo>` instead of mounting
  the repo directly. Build artifacts stay isolated, while commits/snapshots
  flow back through the VCS, not the filesystem.
- Drops you into a terminal multiplexer with Copilot and opens a terminal tab
  in each repo's workspace for committing from the host system

## Prerequisites

- Docker (Docker Desktop on Windows/macOS or native Linux)
- [jj](https://github.com/jj-vcs/jj) and/or [git](https://git-scm.com/)
- A `COPILOT_GITHUB_TOKEN` exported on the host
- Windows terminal (could be abstracted away to support more platforms, but
  since I'm the only user so far, here we go)

## Usage

```
claustro --image <DOCKERFILE_DIR> <REPO>... [-- <copilot args>]
```

Example using the bundled reference image:

```
claustro --image ./images/claustro-martin ./my-project ./other-repo
```

Each `<REPO>` must be the root of a jj or git repository. Claustro will refuse
anything else.

### Flags

- `--image <DIR>` directory containing a `Dockerfile`. The image is tagged
  `<directory-basename>:latest`
- `-d`, `--debug` show docker and VCS subprocess output during startup
  (otherwise hidden behind a spinner)
- `--` everything after is forwarded verbatim to `copilot` inside the
  container

### Environment

- `JJ_BINARY` path to the `jj` binary if not on PATH
- `GIT_BINARY` path to the `git` binary if not on PATH

## Writing your own image

Install whatever tools you want Copilot to have access to. Claustro adds the
entrypoint, the `copilot` user, the workspace setup, and zellij as a wrapper
layer. See
[images/claustro-martin/Dockerfile](images/claustro-martin/Dockerfile) for a
reference.

## How cleanup works

When the session ends, claustro:

- Snapshots any pending changes Copilot made in each workspace (jj auto-snapshot
  via `jj status`, or `git stash push --include-untracked`)
- Removes the workspace from the source repo (`jj workspace forget` /
  `git worktree remove --force`)
- Deletes the temp directory

Your host repo is never touched directly. All changes are recoverable through
the VCS.

## License
Licensed under either of

 * [Apache License, Version 2.0](LICENSE-APACHE)
 * [MIT license](LICENSE-MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
