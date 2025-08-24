#[doc(hidden)]
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiUserKeyResult {
    #[serde(rename = "apiKey")]
    pub api_key: String,
}
