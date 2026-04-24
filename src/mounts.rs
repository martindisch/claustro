use eyre::{Result, WrapErr, eyre};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct ResolvedMount {
    pub directory_name: String,
    pub host_path: PathBuf,
}

pub fn resolve(paths: &[PathBuf]) -> Result<Vec<ResolvedMount>> {
    let mut resolved_directories: HashMap<String, PathBuf> = HashMap::new();

    for path in paths {
        if !path.exists() {
            return Err(eyre!("Mount path does not exist: {}", path.display()));
        }
        if !path.is_dir() {
            return Err(eyre!("Mount path is not a directory: {}", path.display()));
        }

        let canonical_path = path
            .canonicalize()
            .wrap_err_with(|| format!("Canonicalizing {}", path.display()))?;
        let directory_name = canonical_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| {
                eyre!(
                    "Cannot derive directory name from {}",
                    canonical_path.display()
                )
            })?
            .to_string();

        if let Some(prev) = resolved_directories.get(&directory_name) {
            return Err(eyre!(
                "Mount basename collision on '{}': {} and {}",
                directory_name,
                prev.display(),
                canonical_path.display()
            ));
        }

        resolved_directories.insert(directory_name, canonical_path);
    }

    let resolved_mounts = resolved_directories
        .into_iter()
        .map(|(directory_name, host_path)| ResolvedMount {
            host_path,
            directory_name,
        })
        .collect();

    Ok(resolved_mounts)
}

/// Convert a (canonicalized) host path into a form Docker accepts as a bind-mount source.
/// On Windows, `canonicalize` returns UNC paths like `\\?\C:\...`; strip the prefix.
pub fn to_docker_source(path: &Path) -> String {
    let stringified_path = path.to_string_lossy().to_string();

    stringified_path
        .strip_prefix(r"\\?\")
        .map(str::to_string)
        .unwrap_or(stringified_path)
}
