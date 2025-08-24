#[doc(hidden)]
use reqwest::Body;

#[doc(hidden)]
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ProvisioningRequest {
    #[serde(rename = "providerCallbackHost")]
    pub provider_callback_host: String,
}

impl From<ProvisioningRequest> for Body {
    fn from(provisioning_request: ProvisioningRequest) -> Self {
        Body::from(serde_json::to_string(&provisioning_request).unwrap())
    }
}
