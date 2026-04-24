use eyre::{Result, WrapErr, eyre};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

const CREDENTIALS_FILENAME: &str = ".credentials.json";
const SETTINGS_FILENAME: &str = "settings.json";
const USER_CONFIG_FILENAME: &str = ".claude.json";

pub struct SessionDirectory {
    temp: TempDir,
    pub claude_dir: PathBuf,
    pub user_config: Option<PathBuf>,
}

impl SessionDirectory {
    pub fn path(&self) -> &Path {
        self.temp.path()
    }
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

    let host_settings = host_claude_dir.join(SETTINGS_FILENAME);
    if host_settings.is_file() {
        copy_into(&host_settings, &claude_dir.join(SETTINGS_FILENAME))?;
    }

    let host_user_config = home.join(USER_CONFIG_FILENAME);
    let user_config = if host_user_config.is_file() {
        let dest = temp.path().join(USER_CONFIG_FILENAME);
        copy_into(&host_user_config, &dest)?;
        Some(dest)
    } else {
        None
    };

    Ok(SessionDirectory {
        temp,
        claude_dir,
        user_config,
    })
}

fn copy_into(src: &Path, dest: &Path) -> Result<()> {
    fs::copy(src, dest)
        .wrap_err_with(|| format!("Copying {} to {}", src.display(), dest.display()))?;
    Ok(())
}
