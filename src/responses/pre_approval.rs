#[doc(hidden)]
use serde::{Deserialize, Serialize};

use crate::structs::party::Party;

#[derive(Debug, Serialize, Deserialize)]
pub struct PreApprovalResult {
    pub payer: Party,
    #[serde(rename = "payerCurrency")]
    pub payer_currency: String,
    pub status: String,
    #[serde(rename = "expirationDateTime")]
    pub expiration_date_time: String,
}
