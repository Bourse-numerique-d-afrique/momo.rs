use serde::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfoWithConsent {
    pub sub: String,
    pub name: String,
    #[serde(rename = "givenName")]
    pub given_name: String,
    #[serde(rename = "familyName")]
    pub family_name: String,
    #[serde(rename = "middleName")]
    pub middle_name: String,
    pub email: String,
    #[serde(rename = "emailVerified")]
    pub email_verified: bool,
    pub gender: String,
    pub locale: String,
    #[serde(rename = "phoneNumber")]
    pub phone_number: String,
    #[serde(rename = "phoneNumberVerified")]
    pub phone_number_verified: bool,
    pub address: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: i32,
    pub status: String,
    pub birthdate: String,
    #[serde(rename = "creditScore")]
    pub credit_score: String,
    pub active: bool,
    #[serde(rename = "countryOfBirth")]
    pub country_of_birth: String,
    #[serde(rename = "regionOfBirth")]
    pub region_of_birth: String,
    #[serde(rename = "cityOfBirth")]
    pub city_of_birth: String,
    pub occupation: String,
    #[serde(rename = "employerName")]
    pub employer_name: String,
    #[serde(rename = "identificationType")]
    pub identification_type: String,
    #[serde(rename = "identificationValue")]
    pub identification_value: String,
}