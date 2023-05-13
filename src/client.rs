use twitch_api::{twitch_oauth2::UserToken, HelixClient};

pub struct Client {
    pub helix: HelixClient<'static, reqwest::Client>,
    pub token: UserToken,
}

impl Client {
    // IMPORTANT: Careful not to expose `secrets` in the error message.
    pub async fn new(secrets: &crate::secret::Secrets) -> color_eyre::Result<Client> {
        // Create the HelixClient, which is used to make requests to the Twitch API
        let helix: HelixClient<reqwest::Client> = HelixClient::default();

        let token = crate::token::get_user_token(secrets, &helix).await?;

        Ok(Client { helix, token })
    }
}
