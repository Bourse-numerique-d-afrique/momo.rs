
use reqwest::Body;
use serde::{Serialize, Deserialize};

use crate::{structs::party::Party, enums::{currency::Currency, payer_identification_type::PayerIdentificationType}};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CashTransferRequest {
    pub amount: String,
    pub currency: Currency,
    pub payee: Party,
    #[serde(rename = "externalId")]
    pub external_id: String,
    #[serde(rename = "orginatingCountry")]
    pub originating_country: String,
    #[serde(rename = "originalAmount")]
    pub original_amount: String,
    #[serde(rename = "originalCurrency")]
    pub original_currency: Currency,
    #[serde(rename = "payerMessage")]
    pub payer_message: String,
    #[serde(rename = "payeeNote")]
    pub payee_note: String,
    #[serde(rename = "payerIdentificationType")]
    pub payer_identification_type: PayerIdentificationType,
    #[serde(rename = "payerIdentificationNumber")]
    pub payer_identification_number: String,
    #[serde(rename = "payerIdentity")]
    pub payer_identity: String,
    #[serde(rename = "payerFirstName")]
    pub payer_first_name: String,
    #[serde(rename = "payerSurName")]
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


impl CashTransferRequest {
    pub fn new(amount: String, currency: Currency, payee: Party, originating_country: String, original_amount: String,
         original_currency: Currency, payer_message: String, payee_note: String, payer_identification_type: PayerIdentificationType, payer_identification_number: String, 
         payer_identity: String, payer_first_name: String, payer_surname: String, payer_language_code: String, payer_email: String, payer_msisdn: String, payer_gender: String) -> Self{
        let external_id = uuid::Uuid::new_v4().to_string();
        Self { amount, currency, payee, external_id, originating_country, original_amount, original_currency, payer_message, payee_note, payer_identification_type, payer_identification_number, payer_identity, 
            payer_first_name, payer_surname, payer_language_code, payer_email, payer_msisdn, payer_gender }

    }
}




impl From<CashTransferRequest> for Body {
    fn from(cash_transfer_request: CashTransferRequest) -> Self {
        Body::from(serde_json::to_string(&cash_transfer_request).unwrap())
    }
}