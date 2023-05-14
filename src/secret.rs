use color_eyre::eyre::Context;
use serde::Deserialize;
use tracing::instrument;

pub struct Secrets {
    // pub path: std::path::PathBuf,
    pub client: SecretClient,
}

#[derive(Deserialize)]
pub struct SecretClient {
    pub login_name: String,
    pub oauth_token: String,
    // pub client_id: String,
    // pub client_secret: String,
}

impl Secrets {
    /// Loads the secrets from the default secrets folder.
    #[instrument]
    pub fn load() -> color_eyre::Result<Self> {
        Self::load_from("secrets")
    }

    /// Loads the secrets from the given secrets folder.
    pub fn load_from(path: impl AsRef<std::path::Path>) -> color_eyre::Result<Self> {
        let path = path.as_ref().to_owned();

        let client = crate::util::fs::read_toml(path.join("login.toml"))
            .wrap_err("when loading client secret")?;

        Ok(Self { client })
    }
}
