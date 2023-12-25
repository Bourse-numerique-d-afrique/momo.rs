


use serde::{Serialize, Deserialize};
use reqwest::Body;

use crate::structs::party::Party;

#[derive(Debug, Serialize, Deserialize)]
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

impl From<Transfer> for Body {
    fn from(transfer: Transfer) -> Self {
        Body::from(serde_json::to_string(&transfer).unwrap())
    }
}