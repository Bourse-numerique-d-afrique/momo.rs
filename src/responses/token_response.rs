#[doc(hidden)]
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};


#[derive(Debug, Serialize,  Clone)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i32,
    pub created_at: Option<DateTime<Utc>>
}


impl<'de> Deserialize<'de> for TokenResponse {
    fn deserialize<D>(deserializer: D) -> Result<TokenResponse, D::Error> where D: serde::Deserializer<'de> {
        let mut map = serde_json::Map::deserialize(deserializer)?;
        let access_token = map.remove("access_token")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .ok_or_else(|| serde::de::Error::missing_field("access_token"))?;
    let token_type = map.remove("token_type")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .ok_or_else(|| serde::de::Error::missing_field("token_type"))?;
    let expires_in = map.remove("expires_in")
        .and_then(|v| v.as_i64().map(|i| i as i32))
        .ok_or_else(|| serde::de::Error::missing_field("expires_in"))?;
        let created_at = Some(Utc::now());
        Ok(TokenResponse {
            access_token,
            token_type,
            expires_in,
            created_at,
        })
    }
}


