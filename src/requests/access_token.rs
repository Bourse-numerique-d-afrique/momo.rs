
use reqwest::Body;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccessTokenRequest{
    pub grant_type: String,
    pub auth_req_id: String,
}


impl From<AccessTokenRequest> for Body{
    fn from(access_token_request: AccessTokenRequest) -> Self{
        Body::from(serde_json::to_string(&access_token_request).unwrap())
    }
}