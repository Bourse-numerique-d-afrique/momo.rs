
use reqwest::Body;
use serde::{Serialize, Deserialize};

use crate::structs::money::Money;


#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePayment {
    #[serde(rename = "externalTransactionId")]
    pub external_transaction_id: String,
    pub money: Money,
    #[serde(rename = "customerReference")]
    pub customer_reference: String,
    #[serde(rename = "serviceProviderUserName")]
    pub service_provider_user_name: String,
    #[serde(rename = "couponId")]
    pub coupon_id: String,
    #[serde(rename = "productId")]
    pub product_id: String,
    #[serde(rename = "productOfferingId")]
    pub product_offering_id: String,
    #[serde(rename = "receiverMessage")]
    pub receiver_message: String,
    #[serde(rename = "senderNote")]
    pub sender_note: String,
    #[serde(rename = "maxNumberOfRetries")]
    pub max_number_of_retries: i32,
    #[serde(rename = "includeSenderCharges")]
    pub include_sender_charges: bool,
}


impl From<CreatePayment> for Body {
    fn from(create_payment: CreatePayment) -> Self {
        Body::from(serde_json::to_string(&create_payment).unwrap())
    }
}