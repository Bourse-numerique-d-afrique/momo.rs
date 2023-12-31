use crate::{responses::account_info::BasicUserInfoJsonResponse, structs::balance::Balance, Currency};

pub trait Account {
    fn get_account_balance(&self) -> impl std::future::Future<Output = Result<Balance, Box<dyn std::error::Error>>> + Send;
    fn get_account_balance_in_specific_currency(&self, currency: Currency) -> impl std::future::Future<Output = Result<Balance, Box<dyn std::error::Error>>> + Send;
    fn get_basic_user_info(&self, account_holder_msisdn: &str) ->impl std::future::Future<Output = Result<BasicUserInfoJsonResponse, Box<dyn std::error::Error>>> + Send;
    fn get_user_info_with_consent(&self, access_token: String) -> impl std::future::Future<Output = Result<BasicUserInfoJsonResponse, Box<dyn std::error::Error>>> + Send;
    fn validate_account_holder_status(&self, account_holder_id: &str, account_holder_type: &str) -> impl std::future::Future<Output = Result<(), Box<dyn std::error::Error>>> + Send;
}