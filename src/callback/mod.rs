use futures_core::Stream;
use std::error::Error;
use tokio::sync::mpsc::{self, Sender};

use crate::enums::{reason::RequestToPayReason, request_to_pay_status::RequestToPayStatus};
use poem::{
    error::ReadBodyError,
    listener::TcpListener,
    middleware::AddData,
    post,
    web::{Data, Path},
    EndpointExt,
};
use serde::{Deserialize, Serialize};

use crate::{CallbackType, Party};
use poem::Result;
use poem::{handler, Route, Server};

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
#[derive(Debug, Serialize, Deserialize)]
pub struct Reason {
    pub code: RequestToPayReason,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum CallbackResponse {
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
        payee_note: String,
        #[serde(rename = "payerMessage")]
        payer_message: String,
        status: RequestToPayStatus,
    },

    // Request to pay failed callback response
    RequestToPayFailed {
        #[serde(rename = "financialTransactionId")]
        financial_transaction_id: String,
        #[serde(rename = "externalId")]
        external_id: String,
        amount: String,
        currency: String,
        payer: Party,
        #[serde(rename = "payeeNote")]
        payee_note: String,
        #[serde(rename = "payerMessage")]
        payer_message: String,
        status: RequestToPayStatus,
        reason: Reason,
    },

    // pre approval success callback response
    PreApprovalSuccess {
        payer: Party,
        #[serde(rename = "payerCurrency")]
        payer_currency: String,
        status: String,
        #[serde(rename = "expirationDateTime")]
        expiration_date_time: String,
    },

    // pre approval failed callback response
    PreApprovalFailed {
        payer: Party,
        #[serde(rename = "payerCurrency")]
        payer_currency: String,
        status: String,
        #[serde(rename = "expirationDateTime")]
        expiration_date_time: String,
        reason: Reason,
    },

    // payment succeded callback response
    PaymentSucceeded {
        #[serde(rename = "referenceId")]
        reference_id: String,
        status: String,
        #[serde(rename = "financialTransactionId")]
        financial_transaction_id: String,
    },

    // paymen failed callback response
    PaymentFailed {
        #[serde(rename = "referenceId")]
        reference_id: String,
        status: String,
        #[serde(rename = "financialTransactionId")]
        financial_transaction_id: String,
        reason: Reason,
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
        payee_note: String,
        #[serde(rename = "payerMessage")]
        payer_message: String,
        status: String,
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
        payee_note: String,
        #[serde(rename = "payerMessage")]
        payer_message: String,
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
        payee_note: String,
        #[serde(rename = "payerMessage")]
        payer_message: String,
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
        payee_note: String,
        #[serde(rename = "payerMessage")]
        payer_message: String,
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
        payee_note: String,
        #[serde(rename = "payerMessage")]
        payer_message: String,
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
        payee_note: String,
        #[serde(rename = "payerMessage")]
        payer_message: String,
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
        payee_note: String,
        #[serde(rename = "payerMessage")]
        payer_message: String,
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
        payee_note: String,
        #[serde(rename = "payerMessage")]
        payer_message: String,
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
        payee_note: String,
        #[serde(rename = "payerMessage")]
        payer_message: String,
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
        payee_note: String,
        #[serde(rename = "payerMessage")]
        payer_message: String,
        status: String,
        reason: Reason,
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
}

pub struct MomoUpdates {
    pub remote_address: String,
    pub response: CallbackResponse,
    pub update_type: CallbackType,
}

#[handler]
async fn mtn_callback(
    req: &poem::Request,
    mut body: poem::Body,
    sender: Data<&Sender<MomoUpdates>>,
    Path(callback_type): Path<String>,
) -> Result<poem::Response, poem::Error> {
    let remote_address = req.remote_addr().clone();
    let string = body.into_string().await?;
    let response_result: Result<CallbackResponse, serde_json::Error> =
        serde_json::from_str(&string);

    match response_result {
        Ok(response) => {
            let momo_updates = MomoUpdates {
                remote_address: remote_address.to_string(),
                response,
                update_type: CallbackType::from_string(&callback_type),
            };

            if let Err(e) = sender.send(momo_updates).await {
                eprintln!("Failed to send callback update: {}", e);
            }
        }
        Err(e) => {
            eprintln!("Failed to parse callback response: {}", e);
        }
    }

    Ok(poem::Response::builder()
        .status(poem::http::StatusCode::OK)
        .body("Callback received successfully"))
}

#[derive(Copy, Clone)]
pub struct MomoCallbackListener;

impl MomoCallbackListener {
    pub async fn serve(port: String) -> Result<impl Stream<Item = MomoUpdates>, Box<dyn Error>> {
        use tracing_subscriber;

        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::TRACE)
            .init();

        let (tx, mut rx) = mpsc::channel::<MomoUpdates>(32);

        std::env::set_var("RUST_BACKTRACE", "1");

        let app = Route::new()
            .at(
                "/collection_request_to_pay/:callback_type",
                post(mtn_callback).put(mtn_callback),
            )
            .at(
                "/collection_request_to_withdraw_v1/:callback_type",
                post(mtn_callback).put(mtn_callback),
            )
            .at(
                "/collection_request_to_withdraw_v2/:callback_type",
                post(mtn_callback).put(mtn_callback),
            )
            .at(
                "/collection_invoice/:callback_type",
                post(mtn_callback).put(mtn_callback),
            )
            .at(
                "/collection_payment/:callback_type",
                post(mtn_callback).put(mtn_callback),
            )
            .at(
                "/collection_preapproval/:callback_type",
                post(mtn_callback).put(mtn_callback),
            )
            .at(
                "/disbursement_deposit_V1/:callback_type",
                post(mtn_callback).put(mtn_callback),
            )
            .at(
                "/disbursement_deposit_v2/:callback_type",
                post(mtn_callback).put(mtn_callback),
            )
            .at(
                "/disburseemnt_refund_v1/:callback_type",
                post(mtn_callback).put(mtn_callback),
            )
            .at(
                "/disburseemnt_refund_v2/:callback_type",
                post(mtn_callback).put(mtn_callback),
            )
            .at(
                "/disburseemnt_transfer/:callback_type",
                post(mtn_callback).put(mtn_callback),
            )
            .at(
                "remittance_cash_transfer/:callback_type",
                post(mtn_callback).put(mtn_callback),
            )
            .at(
                "remittance_transfer/:callback_type",
                post(mtn_callback).put(mtn_callback),
            )
            .with(poem::middleware::Tracing)
            .with(poem::middleware::Cors::new())
            .with(poem::middleware::Compression::default())
            .with(poem::middleware::RequestId::default())
            .with(AddData::new(tx));

        tokio::spawn(async move {
            Server::new(TcpListener::bind(format!("0.0.0.0:{}", port)))
                .run(app)
                .await
                .expect("the server failed to start");
        });

        Ok(async_stream::stream! {
            while let Some(msg) = rx.recv().await {
                yield msg;
            }
        })
    }
}
