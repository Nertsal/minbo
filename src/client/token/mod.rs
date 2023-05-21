mod auth;

use async_trait::async_trait;
use color_eyre::eyre::Context;
use serde::{Deserialize, Serialize};
use twitch_irc::login::{TokenStorage, UserAccessToken};

use crate::secret::Secrets;

const TOKEN_STORAGE: &str = "secrets/token";
const SCOPES: [&str; 3] = ["channel:read:redemptions", "chat:edit", "chat:read"];

#[derive(Serialize, Deserialize, Debug)]
pub struct Scope(String);

impl Scope {
    pub fn new(scope: impl AsRef<str>) -> Self {
        Self(scope.as_ref().to_owned())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CustomTokenStorage {
    pub client_id: String,
    pub client_secret: String,
    pub user_token: UserAccessToken,
}

impl CustomTokenStorage {
    /// Construct from an existing token.
    ///
    /// Call [Self::init] to fetch tokens from file or prompt user.
    fn new(client_id: String, client_secret: String, token: UserAccessToken) -> Self {
        Self {
            client_id,
            client_secret,
            user_token: token,
        }
    }

    /// Fetch access tokens.
    /// Looks in the files first; if no suitable token found, prompts user to authorize.
    pub async fn init(secrets: &Secrets) -> color_eyre::Result<Self> {
        match Self::load(TOKEN_STORAGE) {
            Ok(tokens) => Ok(tokens),
            Err(err) => {
                log::info!("Failed to load tokens from {}: {}", TOKEN_STORAGE, err);
                Self::prompt_user(
                    secrets.client.client_id.clone(),
                    secrets.client.client_secret.clone(),
                )
                .await
            }
        }
    }

    /// Prompt user to authorize and save the token to file.
    async fn prompt_user(client_id: String, client_secret: String) -> color_eyre::Result<Self> {
        log::info!("Prompting user to authorize");

        let scopes: Vec<Scope> = SCOPES.into_iter().map(Scope::new).collect();
        let tokens = auth::authenticate(&client_id, &client_secret, true, &scopes)
            .await
            .wrap_err("when authenticating")?;
        let tokens = Self::new(client_id, client_secret, tokens);

        // Save to file
        crate::util::fs::write_toml(&tokens, TOKEN_STORAGE).wrap_err("when saving tokens")?;

        Ok(tokens)
    }

    /// Attempts to load tokens from the file.
    pub fn load(path: impl AsRef<std::path::Path>) -> color_eyre::Result<Self> {
        crate::util::fs::read_toml(path).wrap_err("when parsing token")
    }
}

#[async_trait]
impl TokenStorage for CustomTokenStorage {
    type LoadError = color_eyre::Report;
    type UpdateError = color_eyre::Report;

    async fn load_token(&mut self) -> Result<UserAccessToken, Self::LoadError> {
        Ok(self.user_token.clone())
    }

    async fn update_token(&mut self, token: &UserAccessToken) -> Result<(), Self::UpdateError> {
        // Save to file
        self.user_token = token.clone();
        crate::util::fs::write_toml(self, TOKEN_STORAGE).wrap_err("when saving tokens")
    }
}
