


use serde::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct BasicUserInfoJsonResponse {
    #[serde(rename = "givenName")]
    pub given_name: String,
    #[serde(rename = "familyName")]
    pub family_name: String,
    pub birthdate: String,
    pub locale: String,
    pub gender: String,
    pub status: String,
}