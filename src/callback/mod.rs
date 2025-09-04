use poem::error::ReadBodyError;
use serde::{Deserialize, Serialize};

use crate::Party;
use crate::enums::reason::RequestToPayReason;

#[derive(thiserror::Error, Debug)]
#[allow(dead_code)]
enum CallbackError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("ReadBody error: {0}")]
    ReadBody(#[from] ReadBodyError),

    #[error("SerdeJson error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("SendError error: {0}")]
    SendError(#[from] Box<tokio::sync::mpsc::error::SendError<MomoUpdates>>),
}

/// MTN momo error Reason
///
/// - 'code', Reason error code
/// - 'message', Reason message
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Reason {
    pub code: RequestToPayReason,
    pub message: String,
}

/// Individual status enums for better parsing
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum RequestToPaySuccessfulStatus {
    #[serde(rename = "SUCCESSFUL")]
    SUCCESSFUL,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum RequestToPayFailedStatus {
    #[serde(rename = "FAILED")]
    FAILED,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum PreApprovalPendingStatus {
    #[serde(rename = "PENDING")]
    PENDING,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum PreApprovalSuccessfulStatus {
    #[serde(rename = "SUCCESSFUL")]
    SUCCESSFUL,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum PreApprovalFailedStatus {
    #[serde(rename = "FAILED")]
    FAILED,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum PreApprovalCreatedStatus {
    #[serde(rename = "CREATED")]
    CREATED,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum PaymentFailedStatus {
    #[serde(rename = "FAILED")]
    FAILED,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum PaymentSucceededStatus {
    #[serde(rename = "SUCCESSFUL")]
    SUCCESSFUL,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum InvoiceFailedStatus {
    #[serde(rename = "FAILED")]
    FAILED,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum InvoiceSucceededStatus {
    #[serde(rename = "SUCCESSFUL")]
    SUCCESSFUL,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum CashTransferFailedStatus {
    #[serde(rename = "FAILED")]
    FAILED,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum CashTransferSucceededStatus {
    #[serde(rename = "SUCCESSFUL")]
    SUCCESSFUL,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum DisbursementDepositV1FailedStatus {
    #[serde(rename = "FAILED")]
    FAILED,
}


#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum DisbursementDepositV2FailedStatus {
    #[serde(rename = "FAILED")]
    FAILED,
}



#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum DisbursementRefundV1FailedStatus {
    #[serde(rename = "FAILED")]
    FAILED,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum DisbursementRefundV1SuccessStatus {
    #[serde(rename = "SUCCESSFUL")]
    SUCCESSFUL,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum DisbursementRefundV2FailedStatus {
    #[serde(rename = "FAILED")]
    FAILED,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum DisbursementRefundV2SuccessStatus {
    #[serde(rename = "SUCCESSFUL")]
    SUCCESSFUL,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum DisbursementFailedStatus {
    #[serde(rename = "FAILED")]
    FAILED,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum DisbursementSuccessStatus {
    #[serde(rename = "SUCCESSFUL")]
    SUCCESSFUL,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum RemittanceTransferFailedStatus {
    #[serde(rename = "FAILED")]
    FAILED,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum RemittanceTransferSuccessStatus {
    #[serde(rename = "SUCCESSFUL")]
    SUCCESSFUL,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum CallbackResponse {
    // Request to pay failed callback response
    RequestToPayFailed {
        #[serde(rename = "financialTransactionId")]
        financial_transaction_id: Option<String>,
        #[serde(rename = "externalId")]
        external_id: String,
        amount: String,
        currency: String,
        payer: Party,
        #[serde(rename = "payeeNote")]
        payee_note: Option<String>,
        #[serde(rename = "payerMessage")]
        payer_message: Option<String>,
        status: RequestToPayFailedStatus,
        reason: RequestToPayReason,
    },

    // Request to pay success callback response
    RequestToPaySuccess {
        #[serde(rename = "financialTransactionId")]
        financial_transaction_id: String,
        #[serde(rename = "externalId")]
        external_id: String,
        amount: String,
        currency: String,
        payer: Party,
        #[serde(rename = "payeeNote")]
        payee_note: Option<String>,
        #[serde(rename = "payerMessage")]
        payer_message: Option<String>,
        status: RequestToPaySuccessfulStatus,
    },


    // pre approval failed callback response
    PreApprovalFailed {
        payer: Party,
        #[serde(rename = "payerCurrency")]
        payer_currency: String,
        #[serde(rename = "expirationDateTime")]
        expiration_date_time: String,
        status: PreApprovalFailedStatus,
        reason: Option<RequestToPayReason>,
    },

    // pre approval success callback response
    PreApprovalSuccess {
        payer: Party,
        #[serde(rename = "payerCurrency")]
        payer_currency: String,
        status: PreApprovalSuccessfulStatus,
        #[serde(rename = "expirationDateTime")]
        expiration_date_time: String,
    },

    // pre approval pending callback response
    PreApprovalPending {
        payer: Party,
        #[serde(rename = "payerCurrency")]
        payer_currency: String,
        status: PreApprovalPendingStatus,
        #[serde(rename = "expirationDateTime")]
        expiration_date_time: String,
    },

    // pre approval created callback response
    PreApprovalCreated {
        payer: Party,
        #[serde(rename = "payerCurrency")]
        payer_currency: String,
        status: PreApprovalCreatedStatus,
        #[serde(rename = "expirationDateTime")]
        expiration_date_time: String,
    },

    // payment failed callback response
    PaymentFailed {
        #[serde(rename = "referenceId")]
        reference_id: String,
        status: PaymentFailedStatus,
        #[serde(rename = "financialTransactionId")]
        financial_transaction_id: Option<String>,
        reason: Reason,
    },

    // payment succeded callback response
    PaymentSucceeded {
        #[serde(rename = "referenceId")]
        reference_id: String,
        status: PaymentSucceededStatus,
        #[serde(rename = "financialTransactionId")]
        financial_transaction_id: Option<String>,
    },

    // invoice failed callback response
    InvoiceFailed {
        #[serde(rename = "referenceId")]
        reference_id: String,
        #[serde(rename = "externalId")]
        external_id: String,
        amount: String,
        currency: String,
        status: InvoiceFailedStatus,
        #[serde(rename = "paymentReference")]
        payment_reference: String,
        #[serde(rename = "invoiceId")]
        invoice_id: String,
        #[serde(rename = "expiryDateTime")]
        expiry_date_time: String,
        #[serde(rename = "intendedPayer")]
        intended_payer: Party,
        description: String,
        #[serde(rename = "errorReason")]
        error_reason: Reason,
    },

    // invoice succeeded callback response
    InvoiceSucceeded {
        #[serde(rename = "referenceId")]
        reference_id: String,
        #[serde(rename = "externalId")]
        external_id: String,
        amount: String,
        currency: String,
        status: InvoiceSucceededStatus,
        #[serde(rename = "paymentReference")]
        payment_reference: String,
        #[serde(rename = "invoiceId")]
        invoice_id: String,
        #[serde(rename = "expiryDateTime")]
        expiry_date_time: String,
        #[serde(rename = "intendedPayer")]
        intended_payer: Party,
        description: String,
    },

    // cash transfer failed callaback response
    CashTransferFailed {
        #[serde(rename = "financialTransactionId")]
        financial_transaction_id: String,
        status: CashTransferFailedStatus,
        reason: String,
        amount: String,
        currency: String,
        payee: Party,
        #[serde(rename = "externalId")]
        external_id: String,
        #[serde(rename = "originatingCountry")]
        originating_country: String,
        #[serde(rename = "originalAmount")]
        original_amount: String,
        #[serde(rename = "originalCurrency")]
        original_currency: String,
        #[serde(rename = "payerMessage")]
        payer_message: String,
        #[serde(rename = "payeeNote")]
        payee_note: String,
        #[serde(rename = "payerIdentificationType")]
        payer_identification_type: String,
        #[serde(rename = "payerIdentificationNumber")]
        payer_identification_number: String,
        #[serde(rename = "payerIdentity")]
        payer_identity: String,
        #[serde(rename = "payerFirstName")]
        payer_first_name: String,
        #[serde(rename = "payerSurname")]
        payer_surname: String,
        #[serde(rename = "payerLanguageCode")]
        payer_language_code: String,
        #[serde(rename = "payerEmail")]
        payer_email: String,
        #[serde(rename = "payerMsisdn")]
        payer_msisdn: String,
        #[serde(rename = "payerGender")]
        payer_gender: String,

        #[serde(rename = "errorReason")]
        error_reason: Reason,
    },

    // cash transfer succeeded callback response
    CashTransferSucceeded {
        #[serde(rename = "financialTransactionId")]
        financial_transaction_id: String,
        status: CashTransferSucceededStatus,
        reason: String,
        amount: String,
        currency: String,
        payee: Party,
        #[serde(rename = "externalId")]
        external_id: String,
        #[serde(rename = "originatingCountry")]
        originating_country: String,
        #[serde(rename = "originalAmount")]
        original_amount: String,
        #[serde(rename = "originalCurrency")]
        original_currency: String,
        #[serde(rename = "payerMessage")]
        payer_message: String,
        #[serde(rename = "payeeNote")]
        payee_note: String,
        #[serde(rename = "payerIdentificationType")]
        payer_identification_type: String,
        #[serde(rename = "payerIdentificationNumber")]
        payer_identification_number: String,
        #[serde(rename = "payerIdentity")]
        payer_identity: String,
        #[serde(rename = "payerFirstName")]
        payer_first_name: String,
        #[serde(rename = "payerSurname")]
        payer_surname: String,
        #[serde(rename = "payerLanguageCode")]
        payer_language_code: String,
        #[serde(rename = "payerEmail")]
        payer_email: String,
        #[serde(rename = "payerMsisdn")]
        payer_msisdn: String,
        #[serde(rename = "payerGender")]
        payer_gender: String,
    },



    // disbursement deposit v1 success callback response
    DisbursementSuccess {
        #[serde(rename = "financialTransactionId")]
        financial_transaction_id: String,
        #[serde(rename = "externalId")]
        external_id: String,
        amount: String,
        currency: String,
        payee: Party,
        #[serde(rename = "payeeNote")]
        payee_note: Option<String>,
        #[serde(rename = "payerMessage")]
        payer_message: Option<String>,
        status: DisbursementSuccessStatus,
    },

    // disbursement deposit v2 failed callback response
    DisbursementDepositV2Failed {
        #[serde(rename = "financialTransactionId")]
        financial_transaction_id: String,
        #[serde(rename = "externalId")]
        external_id: String,
        amount: String,
        currency: String,
        payee: Party,
        #[serde(rename = "payeeNote")]
        payee_note: Option<String>,
        #[serde(rename = "payerMessage")]
        payer_message: Option<String>,
        status: DisbursementDepositV2FailedStatus,
        reason: Option<RequestToPayReason>,
    },



    // disbursement refund v1 failed callback response
    DisbursementRefundV1Failed {
        #[serde(rename = "financialTransactionId")]
        financial_transaction_id: String,
        #[serde(rename = "externalId")]
        external_id: String,
        amount: String,
        currency: String,
        payee: Party,
        #[serde(rename = "payeeNote")]
        payee_note: Option<String>,
        #[serde(rename = "payerMessage")]
        payer_message: Option<String>,
        status: DisbursementRefundV1FailedStatus,
        reason: Option<RequestToPayReason>,
    },

    // disbursement refund v1 success callback response
    DisbursementRefundV1Success {
        #[serde(rename = "financialTransactionId")]
        financial_transaction_id: String,
        #[serde(rename = "externalId")]
        external_id: String,
        amount: String,
        currency: String,
        payee: Party,
        #[serde(rename = "payeeNote")]
        payee_note: Option<String>,
        #[serde(rename = "payerMessage")]
        payer_message: Option<String>,
        status: DisbursementRefundV1SuccessStatus,
    },

    // disbursement refund v2 failed callback response
    DisbursementRefundV2Failed {
        #[serde(rename = "financialTransactionId")]
        financial_transaction_id: String,
        #[serde(rename = "externalId")]
        external_id: String,
        amount: String,
        currency: String,
        payee: Party,
        #[serde(rename = "payeeNote")]
        payee_note: Option<String>,
        #[serde(rename = "payerMessage")]
        payer_message: Option<String>,
        status: DisbursementRefundV2FailedStatus,
        reason: Option<RequestToPayReason>,
    },

    // disbursement refund v2 success callback response
    DisbursementRefundV2Success {
        #[serde(rename = "financialTransactionId")]
        financial_transaction_id: String,
        #[serde(rename = "externalId")]
        external_id: String,
        amount: String,
        currency: String,
        payee: Party,
        #[serde(rename = "payeeNote")]
        payee_note: Option<String>,
        #[serde(rename = "payerMessage")]
        payer_message: Option<String>,
        status: DisbursementRefundV2SuccessStatus,
    },

    // disbursement transfer failed callback response
    DisbursementFailed {
        #[serde(rename = "externalId")]
        external_id: String,
        amount: String,
        currency: String,
        payee: Party,
        #[serde(rename = "payeeNote")]
        payee_note: Option<String>,
        status: DisbursementFailedStatus,
        reason: Option<RequestToPayReason>,
    },

    // remittance transfer failed callback response
    RemittanceTransferFailed {
        #[serde(rename = "financialTransactionId")]
        financial_transaction_id: String,
        status: RemittanceTransferFailedStatus,
        reason: String,
        amount: String,
        currency: String,
        payee: Party,
        #[serde(rename = "externalId")]
        external_id: String,
        #[serde(rename = "originatingCountry")]
        originating_country: String,
        #[serde(rename = "originalAmount")]
        original_amount: String,
        #[serde(rename = "originalCurrency")]
        original_currency: String,
        #[serde(rename = "payerMessage")]
        payer_message: String,
        #[serde(rename = "payeeNote")]
        payee_note: String,
        #[serde(rename = "errorReason")]
        error_reason: Option<RequestToPayReason>,
    },

    // remittance transfer success callback response
    RemittanceTransferSuccess {
        #[serde(rename = "financialTransactionId")]
        financial_transaction_id: String,
        status: RemittanceTransferSuccessStatus,
        reason: String,
        amount: String,
        currency: String,
        payee: Party,
        #[serde(rename = "externalId")]
        external_id: String,
        #[serde(rename = "originatingCountry")]
        originating_country: String,
        #[serde(rename = "originalAmount")]
        original_amount: String,
        #[serde(rename = "originalCurrency")]
        original_currency: String,
        #[serde(rename = "payerMessage")]
        payer_message: String,
        #[serde(rename = "payeeNote")]
        payee_note: String,
    },
}

#[derive(Clone, Debug)]
pub struct MomoUpdates {
    pub remote_address: String,
    pub response: CallbackResponse,
}