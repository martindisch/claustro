use eyre::{Result, WrapErr, eyre};
use serde_json::{Map, Value};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

const CREDENTIALS_FILENAME: &str = ".credentials.json";
const SETTINGS_FILENAME: &str = "settings.json";
const USER_CONFIG_FILENAME: &str = ".claude.json";

pub struct SessionDirectory {
    // Held to keep the temp dir alive for the lifetime of the session.
    _temp: TempDir,
    pub claude_dir: PathBuf,
    pub user_config: PathBuf,
}

pub fn prepare_session_directory() -> Result<SessionDirectory> {
    let home = dirs::home_dir().ok_or_else(|| eyre!("Could not determine home directory"))?;
    let host_claude_dir = home.join(".claude");
    let host_credentials = host_claude_dir.join(CREDENTIALS_FILENAME);

    if !host_credentials.is_file() {
        return Err(eyre!(
            "Credentials file not found at {}. Run `claude` once on the host to log in.",
            host_credentials.display()
        ));
    }

    let temp = tempfile::Builder::new()
        .prefix("claustro-")
        .tempdir()
        .wrap_err("Creating session temp directory")?;

    let claude_dir = temp.path().join(".claude");
    fs::create_dir(&claude_dir).wrap_err_with(|| format!("Creating {}", claude_dir.display()))?;

    copy_into(&host_credentials, &claude_dir.join(CREDENTIALS_FILENAME))?;

    let dest_settings = claude_dir.join(SETTINGS_FILENAME);
    let host_settings = host_claude_dir.join(SETTINGS_FILENAME);
    if host_settings.is_file() {
        copy_into(&host_settings, &dest_settings)?;
    }
    splice_bool_flag(&dest_settings, &["skipDangerousModePermissionPrompt"], true)?;

    let dest_user_config = temp.path().join(USER_CONFIG_FILENAME);
    let host_user_config = home.join(USER_CONFIG_FILENAME);
    if host_user_config.is_file() {
        copy_into(&host_user_config, &dest_user_config)?;
    }
    splice_bool_flag(
        &dest_user_config,
        &["projects", "/workspace", "hasTrustDialogAccepted"],
        true,
    )?;

    Ok(SessionDirectory {
        _temp: temp,
        claude_dir,
        user_config: dest_user_config,
    })
}

fn copy_into(src: &Path, dest: &Path) -> Result<()> {
    fs::copy(src, dest)
        .wrap_err_with(|| format!("Copying {} to {}", src.display(), dest.display()))?;
    Ok(())
}

fn splice_bool_flag(path: &Path, keys: &[&str], value: bool) -> Result<()> {
    let (leaf, parents) = keys
        .split_last()
        .ok_or_else(|| eyre!("Empty key path for {}", path.display()))?;

    let mut json: Value = if path.is_file() {
        let content =
            fs::read_to_string(path).wrap_err_with(|| format!("Reading {}", path.display()))?;
        serde_json::from_str(&content).wrap_err_with(|| format!("Parsing {}", path.display()))?
    } else {
        Value::Object(Map::new())
    };

    let mut current = &mut json;
    for key in parents {
        let map = match current {
            Value::Object(m) => m,
            _ => {
                return Err(eyre!(
                    "Expected JSON object at '{}' in {}",
                    key,
                    path.display()
                ));
            }
        };
        current = map
            .entry(key.to_string())
            .or_insert_with(|| Value::Object(Map::new()));
    }

    match current {
        Value::Object(map) => {
            map.insert(leaf.to_string(), Value::Bool(value));
        }
        _ => {
            return Err(eyre!(
                "Expected JSON object at leaf parent in {}",
                path.display()
            ));
        }
    }

    let serialized = serde_json::to_string_pretty(&json)
        .wrap_err_with(|| format!("Serializing {}", path.display()))?;
    fs::write(path, serialized).wrap_err_with(|| format!("Writing {}", path.display()))?;

    Ok(())
}
