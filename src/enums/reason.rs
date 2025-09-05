#[doc(hidden)]
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub enum RequestToPayReason {
    #[serde(rename = "INTERNAL_PROCESSING_ERROR")]
    InternalProcessingError,
    #[serde(rename = "APPROVAL_REJECTED")]
    APPROVALREJECTED,
    EXPIRED,
    ONGOING,
    #[serde(rename = "PAYER_DELAYED")]
    PAYERDELAYED,
    #[serde(rename = "PAYER_NOT_FOUND")]
    PAYERNOTFOUND,
    #[serde(rename = "PAYEE_NOT_ALLOWED_TO_RECEIVE")]
    PAYEENOTALLOWEDTORECEIVE,
    #[serde(rename = "NOT_ALLOWED")]
    NOTALLOWED,
    #[serde(rename = "NOT_ALLOWED_TARGET_ENVIRONMENT")]
    NOTALLOWEDTARGETENVIRONMENT,
    #[serde(rename = "INVALID_CALLBACK_URL_HOST")]
    INVALIDCALLBACKURLHOST,
    #[serde(rename = "INVALID_CURRENCY")]
    INVALIDCURRENCY,
    #[serde(rename = "SERVICE_UNAVAILABLE")]
    SERVICEUNAVAILABLE,
    #[serde(rename = "COULD_NOT_PERFORM_TRANSACTION")]
    COULDNOTPERFORMTRANSACTION,
    #[serde(rename = "NOT_ENOUGH_FUNDS")]
    NOTENOUGHFUNDS,
    #[serde(rename = "PAYEE_NOT_FOUND")]
    PAYEENOTFOUND,
    #[serde(rename = "PAYER_LIMIT_REACHED")]
    PAYERLIMITREACHED,
    #[serde(rename = "PAYEE_DELAYED")]
    PAYEEDELAYED,
}
