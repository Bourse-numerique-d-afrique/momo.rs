#[doc(hidden)]
use reqwest::Body;

#[doc(hidden)]
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Refund {
    pub amount: String,
    pub currency: String,
    #[serde(rename = "externalId")]
    pub external_id: String,
    #[serde(rename = "payerMessage")]
    pub payer_message: String,
    #[serde(rename = "payeeNote")]
    pub payee_note: String,
    #[serde(rename = "referenceIdToRefund")]
    pub reference_id_to_refund: String,
}

impl Refund {
    pub fn new(
        amount: String,
        currency: String,
        payer_message: String,
        payee_note: String,
        reference_id_to_refund: String,
    ) -> Self {
        let external_id = uuid::Uuid::new_v4().to_string();
        Refund {
            amount,
            currency,
            external_id,
            payer_message,
            payee_note,
            reference_id_to_refund,
        }
    }
}

impl From<Refund> for Body {
    fn from(refund: Refund) -> Self {
        Body::from(serde_json::to_string(&refund).unwrap())
    }
}
