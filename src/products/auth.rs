use crate::{
    AccessTokenRequest, AccessType, BCAuthorizeResponse, BcAuthorizeRequest, Environment,
    OAuth2TokenResponse, TokenResponse,
};

pub struct Authorization {}

impl Authorization {
    /// This operation is used to create an access token
    ///
    /// #Returns
    ///
    /// * 'TokenResponse'
    pub async fn create_access_token(
        &self,
        url: String,
        api_user: String,
        api_key: String,
        primary_key: String,
    ) -> Result<TokenResponse, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let res = client
            .post(format!("{}/token/", url))
            .basic_auth(api_user, Some(api_key))
            .header("Cache-Control", "no-cache")
            .header("Content-type", "application/x-www-form-urlencoded")
            .header("Ocp-Apim-Subscription-Key", &primary_key)
            .header("Content-Length", "0")
            .body("")
            .send()
            .await?;

        if res.status().is_success() {
            let body = res.text().await?;
            let token_response: TokenResponse = serde_json::from_str(&body)?;
            Ok(token_response)
        } else {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                res.text().await?,
            )))
        }
    }

    /// This operation is used to create an OAuth2 token
    ///
    /// #Parameters
    ///
    /// * 'auth_req_id', this is the auth request id
    ///
    /// #Returns
    ///
    /// * 'OAuth2TokenResponse'
    pub async fn create_o_auth_2_token(
        &self,
        url: String,
        api_user: String,
        api_key: String,
        environment: Environment,
        primary_key: String,
        auth_req_id: String,
    ) -> Result<OAuth2TokenResponse, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let res = client
            .post(format!("{}/oauth2/token/", url))
            .basic_auth(api_user.to_string(), Some(api_key.to_string()))
            .header("X-Target-Environment", environment.to_string())
            .header("Content-type", "application/x-www-form-urlencoded")
            .header("Ocp-Apim-Subscription-Key", &primary_key)
            .body(AccessTokenRequest {
                grant_type: "urn:openid:params:grant-type:ciba".to_string(),
                auth_req_id,
            })
            .send()
            .await?;

        if res.status().is_success() {
            let body = res.text().await?;
            let token_response: OAuth2TokenResponse = serde_json::from_str(&body)?;
            Ok(token_response)
        } else {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                res.text().await?,
            )))
        }
    }

    /// This operation is used to authorize a user.
    ///
    /// #Parameters
    /// * 'msisdn', this is the phone number of the user
    /// * 'callback_url', this is the url that will be used to notify the client of the status of the transaction
    ///
    /// #Returns
    ///
    /// * 'BCAuthorizeResponse'
    pub async fn bc_authorize(
        &self,
        url: String,
        environment: Environment,
        primary_key: String,
        msisdn: String,
        callback_url: Option<&str>,
        access_token: TokenResponse,
    ) -> Result<BCAuthorizeResponse, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let mut req = client
            .post(format!("{}/v1_0/bc-authorize", url))
            .bearer_auth(access_token.access_token)
            .header("X-Target-Environment", environment.to_string())
            .header("Content-type", "application/x-www-form-urlencoded")
            .header("Ocp-Apim-Subscription-Key", &primary_key)
            .body(
                BcAuthorizeRequest {
                    login_hint: format!("ID:{}/MSISDN", msisdn),
                    scope: "profile".to_string(),
                    access_type: AccessType::Offline,
                }
                .to_string(),
            );

        if let Some(callback_url) = callback_url {
            if !callback_url.is_empty() {
                req = req.header("X-Callback-Url", callback_url);
            }
        }

        let res = req.send().await?;

        if res.status().is_success() {
            let body = res.text().await?;
            let token_response: BCAuthorizeResponse = serde_json::from_str(&body)?;
            Ok(token_response)
        } else {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                res.text().await?,
            )))
        }
    }
}
