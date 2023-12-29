#[doc(hidden)]
use reqwest::Body;

#[doc(hidden)]
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccessTokenRequest{
    pub grant_type: String,
    pub auth_req_id: String,
}


impl From<AccessTokenRequest> for Body{
    fn from(access_token_request: AccessTokenRequest) -> Self{
        let t =  format!("grant_type={}&auth_req_id={}", access_token_request.grant_type, access_token_request.auth_req_id);
        Body::from(t)
    }
}