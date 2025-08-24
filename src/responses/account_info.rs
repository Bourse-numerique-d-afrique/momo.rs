#[doc(hidden)]
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct BasicUserInfoJsonResponse {
    pub given_name: String,
    pub family_name: String,
    pub birthdate: Option<String>,
    pub locale: Option<String>,
    pub gender: Option<String>,
}
