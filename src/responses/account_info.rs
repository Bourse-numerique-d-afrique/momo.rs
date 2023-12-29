

#[doc(hidden)]
use serde::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct BasicUserInfoJsonResponse {
    pub given_name: String,
    pub family_name: String,
    pub birthdate: String,
    pub locale: String,
    pub gender: String,
}