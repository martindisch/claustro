use eyre::{Result, WrapErr, eyre};
use std::fs;
use tempfile::TempDir;

const CREDENTIALS_FILENAME: &str = ".credentials.json";

pub fn prepare_session_directory() -> Result<TempDir> {
    let home = dirs::home_dir().ok_or_else(|| eyre!("Could not determine home directory"))?;
    let host_credentials = home.join(".claude").join(CREDENTIALS_FILENAME);

    if !host_credentials.is_file() {
        return Err(eyre!(
            "Credentials file not found at {}. Run `claude` once on the host to log in.",
            host_credentials.display()
        ));
    }

    let session_directory = tempfile::Builder::new()
        .prefix("claustro-")
        .tempdir()
        .wrap_err("Creating session temp directory")?;

    let dest = session_directory.path().join(CREDENTIALS_FILENAME);
    fs::copy(&host_credentials, &dest).wrap_err_with(|| {
        format!(
            "Copying credentials from {} to {}",
            host_credentials.display(),
            dest.display()
        )
    })?;

    Ok(session_directory)
}
