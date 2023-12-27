
use reqwest::Body;
use serde::{Serialize, Deserialize};

use crate::structs::party::Party;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CashTransferRequest {
    #[serde(rename = "amount")]
    pub amount: String,
    #[serde(rename = "currency")]
    pub currency: String,
    #[serde(rename = "payee")]
    pub payee: Party,
    #[serde(rename = "externalId")]
    pub external_id: String,
    #[serde(rename = "originatingCountry")]
    pub originating_country: String,
    #[serde(rename = "originalAmount")]
    pub original_amount: String,
    #[serde(rename = "originalCurrency")]
    pub original_currency: String,
    #[serde(rename = "payerMessage")]
    pub payer_message: String,
    #[serde(rename = "payeeNote")]
    pub payee_note: String,
    #[serde(rename = "payerIdentificationType")]
    pub payer_identification_type: String,
    #[serde(rename = "payerIdentificationNumber")]
    pub payer_identification_number: String,
    #[serde(rename = "payerIdentity")]
    pub payer_identity: String,
    #[serde(rename = "payerFirstName")]
    pub payer_first_name: String,
    #[serde(rename = "payerSurname")]
    pub payer_surname: String,
    #[serde(rename = "payerLanguageCode")]
    pub payer_language_code: String,
    #[serde(rename = "payerEmail")]
    pub payer_email: String,
    #[serde(rename = "payerMsisdn")]
    pub payer_msisdn: String,
    #[serde(rename = "payerGender")]
    pub payer_gender: String,
}




impl From<CashTransferRequest> for Body {
    fn from(cash_transfer_request: CashTransferRequest) -> Self {
        Body::from(serde_json::to_string(&cash_transfer_request).unwrap())
    }
}