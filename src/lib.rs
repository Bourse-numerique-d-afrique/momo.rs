//! # MTN Mobile Money Rust package
//! `MTNMomo Rust package` provides for an easy way to connect to MTN MoMo API, it provides for the following products:
//! - Collection
//! - Disbursements
//! - Remittance
//! - Provisioning in case of sandbox environment
//! how to use:
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
//!   let momo = Momo::new_with_provisioning(mtn_url, primary_key.clone()).await.unwrap();
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
//!   let momo = Momo::new_with_provisioning(mtn_url, primary_key.clone()).await.unwrap();
//!   let collection = momo.collection(primary_key, secondary_key);
//!
//!    let payer : Party = Party {
//!           party_id_type: PartyIdType::MSISDN,
//!          party_id: "234553".to_string(),
//!      };
//!
//!   let request = RequestToPay::new("100".to_string(), Currency::EUR, payer, "test_payer_message".to_string(), "test_payee_note".to_string());
//!   let result = collection.request_to_pay(request).await;
//! }
//! ```
//! The above code will request a payment of 100 EUR from the customer with the phone number "234553".
//! The customer will receive a prompt on their phone to confirm the payment.
//! If the customer confirms the payment, the payment will be processed and the customer will receive a confirmation message.
//! If the customer declines the payment, the payment will not be processed and the customer will receive a message informing them that the payment was declined.

#[doc(hidden)]
use std::error::Error;

use poem::{listener::TcpListener, post, EndpointExt};
use uuid::Uuid;

use poem::Result;
#[doc(hidden)]
use poem::{handler, Route, Server};

pub mod enums;
pub mod errors;
pub mod products;
pub mod requests;
pub mod responses;
pub mod structs;

pub type PartyIdType = enums::party_id_type::PartyIdType;
pub type Currency = enums::currency::Currency;
pub type Environment = enums::environment::Environment;
pub type AccessType = enums::access_type::AccessType;

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

pub struct TranserId(String);

impl TranserId {
    pub fn as_string(&self) -> String {
        self.0.clone()
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

pub struct TransactionId(String);

impl TransactionId {
    pub fn as_string(&self) -> String {
        self.0.clone()
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

pub struct RefundId(String);

impl RefundId {
    pub fn as_string(&self) -> String {
        self.0.clone()
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

pub struct InvoiceId(String);

impl InvoiceId {
    pub fn as_string(&self) -> String {
        self.0.clone()
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

pub struct PaymentId(String);

impl PaymentId {
    pub fn as_string(&self) -> String {
        self.0.clone()
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

pub struct WithdrawId(String);

impl WithdrawId {
    pub fn as_string(&self) -> String {
        self.0.clone()
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

pub struct DepositId(String);

impl DepositId {
    pub fn as_string(&self) -> String {
        self.0.clone()
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[handler]
fn mtn_callback(req: &poem::Request) -> poem::Response {
    poem::Response::builder()
        .status(poem::http::StatusCode::OK)
        .body("Callback received successfully")
}

#[handler]
fn mtn_put_calback(req: &poem::Request) -> poem::Response {
    println!("yes put boatch");
    poem::Response::builder()
        .status(poem::http::StatusCode::OK)
        .body("Callback received successfully")
}

pub struct MomoCallbackListener;

impl MomoCallbackListener {
    pub async fn serve(port: String) -> Result<(), Box<dyn Error>> {
        use tracing_subscriber;

        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::TRACE)
            .init();

        std::env::set_var("RUST_BACKTRACE", "1");
        let app = Route::new()
            .at("/mtn", post(mtn_callback).put(mtn_put_calback))
            .with(poem::middleware::Tracing::default())
            .with(poem::middleware::Cors::new())
            .with(poem::middleware::Compression::default())
            .with(poem::middleware::RequestId::default());

        Server::new(TcpListener::bind(format!("0.0.0.0:{}", port)))
            .run(app)
            .await
            .expect("the server failed to start");
        Ok(())
    }
}

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
    ) -> Self {
        Momo {
            url,
            environment,
            api_user,
            api_key: api_key.unwrap(),
        }
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
        let _create_sandbox = provisioning
            .create_sandox(&reference_id, provider_callback_host)
            .await?;
        let api = provisioning.create_api_information(&reference_id).await?;
        return Ok(Momo {
            url,
            environment: Environment::Sandbox,
            api_user: reference_id,
            api_key: api.api_key,
        });
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
            self.environment.clone(),
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
            self.environment.clone(),
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
            self.environment.clone(),
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
