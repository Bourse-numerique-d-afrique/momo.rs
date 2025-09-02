//! # MTN Mobile Money Rust package
//! `MTNMomo Rust package` provides for an easy way to connect to MTN MoMo API, it provides for the following products:
//! - Collection
//! - Disbursements
//! - Remittance
//! - Provisioning in case of sandbox environment
//!   how to use:
//! # Examples
//! ```
//! use mtnmomo::Momo;
//! use uuid::Uuid;
//! use dotenv::dotenv;
//! use std::env;
//!
//! #[tokio::main]
//! async fn main() {
//!   dotenv().ok();
//!   let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set"); // https://sandbox.momodeveloper.mtn.com
//!   let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
//!   let secondary_key = env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
//!   let callback_host = env::var("MTN_CALLBACK_HOST").expect("MTN_CALLBACK_HOST must be set");
//!   let momo = Momo::new_with_provisioning(mtn_url, primary_key.clone(), &callback_host).await.unwrap();
//!   let collection = momo.collection(primary_key, secondary_key);
//! }
//!
//! ```
//! After initializing the Momo struct, you can then use the collection, disbursement or remittance methods to initialize the respective products.
//! The products have methods that you can use to interact with the API.
//! For example, to request a payment from a customer, you can use the request_to_pay method of the Collection product.
//!
//! # Important notes
//! `mtnmomo::Momo::new_with_provisioning` is used to initialize the Momo struct with the sandbox environment.
//!
//! `mtnmomo::Momo::new` is used to initialize the Momo struct with the production environment.
//!
//!
//!
//! # Examples:
//!
//! If you want to request a payment from a customer, you can use the request_to_pay method of the Collection product.
//!
//! ```
//! use mtnmomo::{Momo, Party, PartyIdType, Currency, RequestToPay};
//! use uuid::Uuid;
//! use dotenv::dotenv;
//! use std::env;
//!
//! #[tokio::main]
//! async fn main() {
//!   dotenv().ok();
//!   let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set"); // https://sandbox.momodeveloper.mtn.com
//!   let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
//!   let secondary_key = env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
//!   let callback_host = env::var("MTN_CALLBACK_HOST").expect("MTN_CALLBACK_HOST must be set");
//!   let momo = Momo::new_with_provisioning(mtn_url, primary_key.clone(), &callback_host).await.unwrap();
//!   let collection = momo.collection(primary_key, secondary_key);
//!
//!    let payer : Party = Party {
//!           party_id_type: PartyIdType::MSISDN,
//!          party_id: "234553".to_string(),
//!      };
//!
//!   let request = RequestToPay::new("100".to_string(), Currency::EUR, payer, "test_payer_message".to_string(), "test_payee_note".to_string());
//!   let result = collection.request_to_pay(request, None).await;
//! }
//! ```
//! The above code will request a payment of 100 EUR from the customer with the phone number "234553".
//! The customer will receive a prompt on their phone to confirm the payment.
//! If the customer confirms the payment, the payment will be processed and the customer will receive a confirmation message.
//! If the customer declines the payment, the payment will not be processed and the customer will receive a message informing them that the payment was declined.

use std::error::Error;
use uuid::Uuid;

pub mod callback;
pub mod common;
pub mod enums;
pub mod errors;
pub mod products;
pub mod requests;
pub mod responses;
pub mod structs;

#[cfg(feature = "callback-server")]
pub mod callback_server;

pub type PartyIdType = enums::party_id_type::PartyIdType;
pub type Currency = enums::currency::Currency;
pub type Environment = enums::environment::Environment;
pub type AccessType = enums::access_type::AccessType;
pub type CallbackType = enums::callback_type::CallbackType;

pub type Party = structs::party::Party;
pub type Balance = structs::balance::Balance;
pub type Money = structs::money::Money;

// Requests
pub type RequestToPay = requests::request_to_pay::RequestToPay;
pub type RefundRequest = requests::refund::Refund;
pub type TransferRequest = requests::transfer::Transfer;
pub type CashTransferRequest = requests::cash_transfer::CashTransferRequest;
pub type InvoiceRequest = requests::invoice::InvoiceRequest;
pub type DeleteInvoiceRequest = requests::invoice_delete::InvoiceDelete;
pub type CreatePaymentRequest = requests::create_payment::CreatePayment;
pub type DeliveryNotificationRequest = requests::delivery_notification::DeliveryNotification;
pub type InvoiceDeleteRequest = requests::invoice_delete::InvoiceDelete;
pub type PreApprovalRequest = requests::pre_approval::PreApproval;
pub type BcAuthorizeRequest = requests::bc_authorize::BcAuthorize;
pub type AccessTokenRequest = requests::access_token::AccessTokenRequest;

// Products
pub type MomoCollection = products::collection::Collection;
pub type MomoRemittance = products::remittance::Remittance;
pub type MomoDisbursements = products::disbursements::Disbursements;
pub type MomoProvisioning = products::provisioning::Provisioning;

// Responses
pub type TokenResponse = responses::token_response::TokenResponse;
pub type BCAuthorizeResponse = responses::bcauthorize_response::BCAuthorizeResponse;
pub type OAuth2TokenResponse = responses::oauth2tokenresponse::OAuth2TokenResponse;
pub type BasicUserInfoJsonResponse = responses::account_info::BasicUserInfoJsonResponse;
pub type InvoiceResult = responses::invoice::InvoiceResult;
pub type PaymentResult = responses::payment_result::PaymentResult;
pub type PreApprovalResult = responses::pre_approval::PreApprovalResult;
pub type RequestToPayResult = responses::request_to_pay_result::RequestToPayResult;
pub type CashTransferResult = responses::cash_transfer_result::CashTransferResult;
pub type TransferResult = responses::transfer_result::TransferResult;

// Re-export callback types
pub use callback::{CallbackResponse,  MomoUpdates, Reason};

// Re-export callback server types when feature is enabled
#[cfg(feature = "callback-server")]
pub use callback_server::{start_callback_server, CallbackServerConfig};

// Re-export ID types from common module
pub use common::id::{
    DepositId, InvoiceId, PaymentId, RefundId, TransactionId, TranserId, WithdrawId,
};

#[doc(hidden)]
#[derive(Debug)]
pub struct Momo {
    pub url: String,
    pub environment: Environment,
    pub api_user: String,
    pub api_key: String,
}

impl Momo {
    /// Create a new Momo instance
    /// # Parameters
    /// * 'url' - the url of momo
    /// * 'api_user'
    /// * 'environment' - the environnement of the momo instance SandBox or
    /// * 'api_key' - the api_key
    ///
    pub async fn new(
        url: String,
        api_user: String,
        environment: Environment,
        api_key: Option<String>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let api_key = api_key.ok_or("API key is required")?;
        Ok(Momo {
            url,
            environment,
            api_user,
            api_key,
        })
    }

    /// Create a new Momo instance with provisioning
    /// # Parameters
    /// * 'url' the momo instance url to use
    /// * 'subscription_key' the subscription key to use
    /// * 'provider_callback_host', the callback host that will be used to send momo updates (ex: google.com)
    ///
    /// #Returns
    /// Result<Momo, Box<dyn Error>>
    pub async fn new_with_provisioning(
        url: String,
        subscription_key: String,
        provider_callback_host: &str,
    ) -> Result<Momo, Box<dyn Error>> {
        let provisioning = MomoProvisioning::new(url.clone(), subscription_key.clone());
        let reference_id = Uuid::new_v4().to_string();
        provisioning
            .create_sandox(&reference_id, provider_callback_host)
            .await?;
        let api = provisioning.create_api_information(&reference_id).await?;
        Ok(Momo {
            url,
            environment: Environment::Sandbox,
            api_user: reference_id,
            api_key: api.api_key,
        })
    }

    /// create a new instance of Collection product
    ///
    /// # Parameters
    /// * 'primary_key'
    /// * 'secondary_key'
    ///
    /// # Returns
    ///
    /// * 'MomoCollection', instance of Momo collection product
    pub fn collection(&self, primary_key: String, secondary_key: String) -> MomoCollection {
        MomoCollection::new(
            self.url.clone(),
            self.environment,
            self.api_user.clone(),
            self.api_key.clone(),
            primary_key,
            secondary_key,
        )
    }

    /// create a new instance of Disbursements product
    ///
    /// # Parameters
    /// * 'primary_key'
    /// * 'secondary_key'
    ///
    /// # Returns
    ///
    /// * 'MomoDisbursements', instance of Momo disbursement product
    ///
    pub fn disbursement(&self, primary_key: String, secondary_key: String) -> MomoDisbursements {
        MomoDisbursements::new(
            self.url.clone(),
            self.environment,
            self.api_user.clone(),
            self.api_key.clone(),
            primary_key,
            secondary_key,
        )
    }

    /// create a new instance of Remittance product
    ///
    /// # Parameters
    /// * 'primary_key'
    /// * 'secondary_key'
    ///
    /// # Returns
    ///
    /// * 'MomoRemittance', instance of Momo remittance product
    ///
    ///
    pub fn remittance(&self, primary_key: String, secondary_key: String) -> MomoRemittance {
        MomoRemittance::new(
            self.url.clone(),
            self.environment,
            self.api_user.clone(),
            self.api_key.clone(),
            primary_key,
            secondary_key,
        )
    }
}

#[cfg(test)]
mod tests {
    use dotenv::dotenv;
    use std::env;

    use crate::{Currency, Party, PartyIdType, RequestToPay};

    use super::*;

    #[tokio::test]
    async fn test_collection() {
        dotenv().ok();
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set"); // https://sandbox.momodeveloper.mtn.com
        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key =
            env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let momo = Momo::new_with_provisioning(mtn_url, primary_key.clone(), "test")
            .await
            .unwrap();
        let collection = momo.collection(primary_key, secondary_key);
        assert_eq!(collection.url, "https://sandbox.momodeveloper.mtn.com");
        assert_eq!(collection.environment, Environment::Sandbox);
        let payer: Party = Party {
            party_id_type: PartyIdType::MSISDN,
            party_id: "+242064818006".to_string(),
        };

        let request = RequestToPay::new(
            "100".to_string(),
            Currency::EUR,
            payer,
            "test_payer_message".to_string(),
            "test_payee_note".to_string(),
        );
        let result = collection.request_to_pay(request, None).await;
        assert!(result.is_ok());
    }
}
