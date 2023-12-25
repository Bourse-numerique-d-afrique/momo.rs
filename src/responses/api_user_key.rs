




use serde::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiUserKeyResult {
    #[serde(rename = "apiKey")]
    pub api_key: String,
}