



#[doc(hidden)]
use serde::{Serialize, Deserialize};

use crate::structs::party::Party;


#[derive(Debug, Serialize, Deserialize)]
pub struct TransferResult {
    pub amount : String,
    pub currency : String,
    #[serde(rename = "externalId")]
    pub external_id : String,
    pub payee : Party,
    #[serde(rename = "payerMessage")]
    pub payer_message : String,
    #[serde(rename = "payeeNote")]
    pub payee_note : String,
    pub status : String,
}