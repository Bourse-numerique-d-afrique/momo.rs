use crate::enums::{pre_approval_status::PreApprovalStatus, reason::RequestToPayReason, request_to_pay_status::RequestToPayStatus};
use poem::error::ReadBodyError;
use serde::{Deserialize, Serialize};

use crate::Party;

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
        status: RequestToPayStatus,
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
        status: RequestToPayStatus,
    },

    // pre approval failed callback response
    PreApprovalFailed {
        payer: Party,
        #[serde(rename = "payerCurrency")]
        payer_currency: String,
        status: PreApprovalStatus,
        #[serde(rename = "expirationDateTime")]
        expiration_date_time: String,
        reason: Option<Reason>,
    },

    // pre approval success callback response
    PreApprovalSuccess {
        payer: Party,
        #[serde(rename = "payerCurrency")]
        payer_currency: String,
        status: PreApprovalStatus,
        #[serde(rename = "expirationDateTime")]
        expiration_date_time: String,
    },


    // payment failed callback response
    PaymentFailed {
        #[serde(rename = "referenceId")]
        reference_id: String,
        status: String,
        #[serde(rename = "financialTransactionId")]
        financial_transaction_id: Option<String>,
        reason: Reason,
    },

    // payment succeded callback response
    PaymentSucceeded {
        #[serde(rename = "referenceId")]
        reference_id: String,
        status: String,
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
        status: String,
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
        status: String,
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

    // cash trasnfer failed callaback response
    CashTransferFailed {
        #[serde(rename = "financialTransactionId")]
        financial_transaction_id: String,
        status: String,
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
        status: String,
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

    // disbursement deposit v1 failed callback response
    DisbursementDepositV1Failed {
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
        status: String,
        reason: Reason,
    },

    // disbursement deposit v1 success callback response
    DisbursementDepositV1Success {
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
        status: String,
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
        status: String,
        reason: Reason,
    },

    // disbursement deposit v2 success callback response
    DisbursementDepositV2Success {
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
        status: String,
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
        status: String,
        reason: Reason,
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
        status: String,
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
        status: String,
        reason: Reason,
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
        status: String,
    },

    // disbursement transfer failed callback response
    DisbursementTransferFailed {
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
        status: String,
        reason: Reason,
    },

    // disbursement transfer success callback response
    DisbursementTransferSuccess {
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
        status: String,
    },

    // remittance transfer failed callback response
    RemittanceTransferFailed {
        #[serde(rename = "financialTransactionId")]
        financial_transaction_id: String,
        status: String,
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
        error_reason: Reason,
    },

    // remittance transfer success callback response
    RemittanceTransferSuccess {
        #[serde(rename = "financialTransactionId")]
        financial_transaction_id: String,
        status: String,
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