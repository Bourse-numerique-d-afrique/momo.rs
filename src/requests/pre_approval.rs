


use reqwest::Body;
use serde::{Serialize, Deserialize};

use crate::structs::party::Party;



#[derive(Debug, Serialize, Deserialize)]
pub struct PreApproval {
    pub payer : Party,
    #[serde(rename = "payerCurrency")]
    pub payer_currency : String,
    #[serde(rename = "payerMessage")]
    pub payer_message : String,
    #[serde(rename = "validityTime")]
    pub validity_time : i32,
}


impl From<PreApproval> for Body {
    fn from(pre_approval: PreApproval) -> Self {
        Body::from(serde_json::to_string(&pre_approval).unwrap())
    }
}