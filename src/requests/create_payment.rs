#[doc(hidden)]
use reqwest::Body;

#[doc(hidden)]
use serde::{Deserialize, Serialize};

use crate::structs::money::Money;

#[derive(Debug, Serialize, Deserialize, Clone)]
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

impl CreatePayment {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        money: Money,
        customer_reference: String,
        service_provider_user_name: String,
        coupon_id: String,
        product_id: String,
        product_offering_id: String,
        receiver_message: String,
        sender_note: String,
        max_number_of_retries: i32,
        include_sender_charges: bool,
    ) -> Self {
        let external_id = uuid::Uuid::new_v4().to_string();
        CreatePayment {
            external_transaction_id: external_id,
            money,
            customer_reference,
            service_provider_user_name,
            coupon_id,
            product_id,
            product_offering_id,
            receiver_message,
            sender_note,
            max_number_of_retries,
            include_sender_charges,
        }
    }
}

impl From<CreatePayment> for Body {
    fn from(create_payment: CreatePayment) -> Self {
        Body::from(serde_json::to_string(&create_payment).unwrap())
    }
}
