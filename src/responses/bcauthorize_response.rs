use serde::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct BCAuthorizeResponse {
    #[serde(rename = "authRqId")]
    pub auth_req_id: String,
    pub interval: i64,
    #[serde(rename = "expiresIn")]
    pub expires_in: i64,
}