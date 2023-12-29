

#[doc(hidden)]
use serde::{Serialize, Deserialize};

#[doc(hidden)]
use reqwest::Body;

use crate::structs::party::Party;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transfer {
    pub amount : String,
    pub currency : String,
    #[serde(rename = "externalId")]
    pub external_id : String,
    pub payee : Party,
    #[serde(rename = "payerMessage")]
    pub payer_message : String,
    #[serde(rename = "payeeNote")]
    pub payee_note : String,
}


impl Transfer {
    pub fn new(amount: String, currency: String, payee: Party, payer_message: String, payee_note: String) -> Self {
        let external_id = uuid::Uuid::new_v4().to_string();
        Transfer {
            amount,
            currency,
            external_id,
            payee,
            payer_message,
            payee_note
        }
    }
}

impl From<Transfer> for Body {
    fn from(transfer: Transfer) -> Self {
        Body::from(serde_json::to_string(&transfer).unwrap())
    }
}