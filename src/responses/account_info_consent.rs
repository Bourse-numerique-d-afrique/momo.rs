#[doc(hidden)]
use serde::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfoWithConsent {
    pub sub: String,
    pub name: String,
    pub given_name: String,
    pub family_name: String,
    pub middle_name: String,
    pub email: String,
    pub email_verified: bool,
    pub gender: String,
    pub locale: String,
    pub phone_number: String,
    pub phone_number_verified: bool,
    pub address: String,
    pub updated_at: i32,
    pub status: String,
    pub birthdate: String,
    pub credit_score: String,
    pub active: bool,
    pub country_of_birth: String,
    pub region_of_birth: String,
    pub city_of_birth: String,
    pub occupation: String,
    pub employer_name: String,
    pub identification_type: String,
    pub identification_value: String,
}