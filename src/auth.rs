use eyre::{Result, eyre};

pub const COPILOT_TOKEN_ENV: &str = "COPILOT_GITHUB_TOKEN";

/// Reads the GitHub Copilot token from the host environment so it can be
/// forwarded into the container under the same variable name.
pub fn read_copilot_token() -> Result<String> {
    let token = std::env::var(COPILOT_TOKEN_ENV).map_err(|_| {
        eyre!(
            "{COPILOT_TOKEN_ENV} is not set. Export it on the host so the container can authenticate with GitHub Copilot."
        )
    })?;

    if token.trim().is_empty() {
        return Err(eyre!("{COPILOT_TOKEN_ENV} is set but empty."));
    }

    Ok(token)
}
