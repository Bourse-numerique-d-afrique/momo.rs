

use reqwest::Body;
use serde::{Serialize, Deserialize};

use crate::{structs::party::Party, enums::currency::Currency};


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RequestToPay {
    pub amount : String, // Amount that will be debited from the payer account.
    pub currency : Currency, // ISO4217 Currency
    /*
    External id is used as a reference to the transaction.
    External id is used for reconciliation. The external id will be included in transaction history report.
    External id is not required to be unique.
     */
    #[serde(rename = "externalId")]
    pub external_id : String,
    pub payer : Party,
    #[serde(rename = "payerMessage")]
    pub payer_message : String, // Message that will be written in the payer transaction history message field.
    #[serde(rename = "payeeNote")]
    pub payee_note : String // Message that will be written in the payee transaction history note field.

}

impl RequestToPay {

    pub fn new(amount: String, currency: Currency, payer: Party, payer_message: String, payee_note: String) -> Self {
        let external_id = uuid::Uuid::new_v4().to_string();
        RequestToPay {
            amount,
            currency,
            external_id,
            payer,
            payer_message,
            payee_note
        }
    }
}


impl From<RequestToPay> for Body {
    fn from(request_to_pay: RequestToPay) -> Self {
        Body::from(serde_json::to_string(&request_to_pay).unwrap())
    }
}