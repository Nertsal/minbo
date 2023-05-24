use color_eyre::eyre::Context;
use serde::Deserialize;

pub struct Secrets {
    pub client: SecretClient,
}

#[derive(Deserialize)]
pub struct SecretClient {
    pub client_id: String,
    pub client_secret: String,
}

impl Secrets {
    /// Loads the secrets from the given secrets folder.
    pub fn load(path: impl AsRef<std::path::Path>) -> color_eyre::Result<Self> {
        let path = path.as_ref().to_owned();

        let client = crate::util::fs::read_toml(path.join("login.toml"))
            .wrap_err("when loading client secret")?;

        Ok(Self { client })
    }
}
