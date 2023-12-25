
use serde::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct OAuth2TokenResponse {
    pub access_token: String,
    #[serde(rename = "tokenType")]
    pub token_type: String,
    #[serde(rename = "expiresIn")]
    pub expires_in: i64,
    pub scope: String,
    #[serde(rename = "refreshToken")]
    pub refresh_token: String,
    #[serde(rename = "refreshTokenExpiresIn")]
    pub refresh_token_experired_in: i64,
}