use eyre::{Result, WrapErr, eyre};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct ResolvedMount {
    pub host_path: PathBuf,
    pub basename: String,
}

pub fn resolve(inputs: &[PathBuf]) -> Result<Vec<ResolvedMount>> {
    let mut resolved = Vec::with_capacity(inputs.len());
    let mut seen: HashMap<String, PathBuf> = HashMap::new();

    for input in inputs {
        if !input.exists() {
            return Err(eyre!("Mount path does not exist: {}", input.display()));
        }
        if !input.is_dir() {
            return Err(eyre!("Mount path is not a directory: {}", input.display()));
        }
        let canonical = input
            .canonicalize()
            .wrap_err_with(|| format!("Canonicalizing {}", input.display()))?;
        let basename = canonical
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| eyre!("Cannot derive basename from {}", canonical.display()))?
            .to_string();

        if let Some(prev) = seen.get(&basename) {
            return Err(eyre!(
                "Mount basename collision on '{}': {} and {}",
                basename,
                prev.display(),
                canonical.display()
            ));
        }
        seen.insert(basename.clone(), canonical.clone());

        resolved.push(ResolvedMount {
            host_path: canonical,
            basename,
        });
    }

    Ok(resolved)
}

/// Convert a (canonicalized) host path into a form Docker accepts as a bind-mount source.
/// On Windows, `canonicalize` returns UNC paths like `\\?\C:\...`; strip the prefix.
pub fn to_docker_source(path: &Path) -> String {
    let s = path.to_string_lossy().to_string();
    s.strip_prefix(r"\\?\").map(str::to_string).unwrap_or(s)
}
