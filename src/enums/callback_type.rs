#[doc(hidden)]
use std::fmt;

#[doc(hidden)]
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub enum CallbackType {
    #[serde(rename = "REQUEST_TO_PAY")]
    RequestToPay,

    #[serde(rename = "REQUEST_TO_WITHDRAW_V1")]
    RequestToWithdrawV1,

    #[serde(rename = "REQUEST_TO_WITHDRAW_V2")]
    RequestToWithdrawV2,

    #[serde(rename = "INVOICE")]
    Invoice,

    #[serde(rename = "COLLECTION_PAYMENT")]
    CollectionPayment,

    #[serde(rename = "COLLECTION_PRE_APPROVAL")]
    CollectionPreApproval,

    #[serde(rename = "DISBURSEMENT_DEPOSIT_V1")]
    DisbursementDepositV1,

    #[serde(rename = "DISBURSEMENT_DEPOSIT_V2")]
    DisbursementDepositV2,

    #[serde(rename = "DISBURSEMENT_REFUND_V1")]
    DisbursementRefundV1,

    #[serde(rename = "DISBURSEMENT_REFUND_V2")]
    DisbursementRefundV2,

    #[serde(rename = "DISBURSEMENT_TRANSFER")]
    DisbusrementTransfer,

    #[serde(rename = "REMITTANCE_CASH_TRANSFER")]
    RemittanceCashTransfer,

    #[serde(rename = "REMITTANCE_TRANSFER")]
    RemittanceTransfer,

    #[serde(rename = "None")]
    None,
}

impl CallbackType {
    pub fn from_string(s: &str) -> CallbackType {
        match s {
            "REQUEST_TO_PAY" => CallbackType::RequestToPay,
            "REQUEST_TO_WITHDRAW_V1" => CallbackType::RequestToWithdrawV1,
            "REQUEST_TO_WITHDRAW_V2" => CallbackType::RequestToWithdrawV2,
            "INVOICE" => CallbackType::Invoice,
            "COLLECTION_PAYMENT" => CallbackType::CollectionPayment,
            "COLLECTION_PRE_APPROVAL" => CallbackType::CollectionPreApproval,
            "DISBURSEMENT_DEPOSIT_V1" => CallbackType::DisbursementDepositV1,
            "DISBURSEMENT_DEPOSIT_V2" => CallbackType::DisbursementDepositV2,
            "DISBURSEMENT_REFUND_V1" => CallbackType::DisbursementRefundV1,
            "DISBURSEMENT_REFUND_V2" => CallbackType::DisbursementRefundV2,
            "DISBURSEMENT_TRANSFER" => CallbackType::DisbusrementTransfer,
            "REMITTANCE_CASH_TRANSFER" => CallbackType::RemittanceCashTransfer,
            "REMITTANCE_TRANSFER" => CallbackType::RemittanceTransfer,
            _ => CallbackType::None,
        }
    }
}

impl fmt::Display for CallbackType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CallbackType::RequestToPay => write!(f, "REQUEST_TO_PAY"),
            CallbackType::RequestToWithdrawV1 => write!(f, "REQUEST_TO_WITHDRAW_V1"),
            CallbackType::RequestToWithdrawV2 => write!(f, "REQUEST_TO_WITHDRAW_V2"),
            CallbackType::Invoice => write!(f, "INVOICE"),
            CallbackType::CollectionPayment => write!(f, "COLLECTION_PAYMENT"),
            CallbackType::CollectionPreApproval => write!(f, "COLLECTION_PRE_APPROVAL"),
            CallbackType::DisbursementDepositV1 => write!(f, "DISBURSEMENT_DEPOSIT_V1"),
            CallbackType::DisbursementDepositV2 => write!(f, "DISBURSEMENT_DEPOSIT_V2"),
            CallbackType::DisbursementRefundV1 => write!(f, "DISBURSEMENT_REFUND_V1"),
            CallbackType::DisbursementRefundV2 => write!(f, "DISBURSEMENT_REFUND_V2"),
            CallbackType::DisbusrementTransfer => write!(f, "DISBURSEMENT_TRANSFER"),
            CallbackType::RemittanceCashTransfer => write!(f, "REMITTANCE_CASH_TRANSFER"),
            CallbackType::RemittanceTransfer => write!(f, "REMITTANCE_TRANSFER"),
            CallbackType::None => write!(f, "NONE"),
        }
    }
}
