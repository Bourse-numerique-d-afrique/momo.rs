use serde::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct BCAuthorizeResponse {
    pub auth_req_id: String,
    pub interval: i64,
    pub expires_in: i64,
}