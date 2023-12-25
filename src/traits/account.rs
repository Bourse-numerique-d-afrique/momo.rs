use crate::{responses::{account_info::BasicUserInfoJsonResponse, account_info_consent::UserInfoWithConsent}, structs::balance::Balance};

pub trait Account {
    async fn get_account_balance(&self) -> Result<Balance, Box<dyn std::error::Error>>;
    async fn get_account_balance_in_specific_currency(&self, currency: String) -> Result<Balance, Box<dyn std::error::Error>>;
    async fn get_basic_user_info(&self) -> Result<BasicUserInfoJsonResponse, Box<dyn std::error::Error>>;
    async fn get_user_info_with_consent(&self) -> Result<UserInfoWithConsent, Box<dyn std::error::Error>>;
    async fn validate_account_holder_status(&self, account_holder_id: String) -> Result<(), Box<dyn std::error::Error>>;
}