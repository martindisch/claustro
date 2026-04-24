use eyre::{Result, WrapErr, eyre};
use std::fs;
use tempfile::TempDir;

const CREDENTIALS_FILENAME: &str = ".credentials.json";

pub fn prepare_session() -> Result<TempDir> {
    let home = dirs::home_dir().ok_or_else(|| eyre!("Could not determine home directory"))?;
    let host_creds = home.join(".claude").join(CREDENTIALS_FILENAME);

    if !host_creds.is_file() {
        return Err(eyre!(
            "Credentials file not found at {}. Run `claude` once on the host to log in.",
            host_creds.display()
        ));
    }

    let session = tempfile::Builder::new()
        .prefix("claustro-")
        .tempdir()
        .wrap_err("Creating session temp directory")?;

    let dest = session.path().join(CREDENTIALS_FILENAME);
    fs::copy(&host_creds, &dest).wrap_err_with(|| {
        format!(
            "Copying credentials from {} to {}",
            host_creds.display(),
            dest.display()
        )
    })?;

    Ok(session)
}
