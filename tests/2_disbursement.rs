mod common;

#[cfg(test)]
mod tests {
    use crate::common::CallbackTestHelper;
    use futures_util::StreamExt;
    use mtnmomo::{
        enums::reason::RequestToPayReason,
        callback::{
            DisbursementDepositV1SuccessStatus, DisbursementDepositV1FailedStatus,
            DisbursementTransferSuccessStatus, DisbursementTransferFailedStatus,
            DisbursementRefundV1FailedStatus,
        },
        Currency, Momo, Party, PartyIdType, RefundRequest,
        TransferRequest,
    };
    use std::env;
    use tokio::sync::OnceCell;

    static MOMO: OnceCell<Momo> = OnceCell::const_new();

    async fn get_momo() -> &'static Momo {
        MOMO.get_or_init(|| async {
            let mtn_url = env::var("MTN_URL").expect("MTN_URL must be set");
            let subscription_key =
                env::var("MTN_DISBURSEMENT_PRIMARY_KEY").expect("PRIMARY_KEY must be set");

            let callback_host =
                env::var("MTN_CALLBACK_HOST").unwrap_or_else(|_| "webhook.site".to_string());

            println!("Using MTN_CALLBACK_HOST: {}", callback_host);
            let momo_result =
                Momo::new_with_provisioning(mtn_url, subscription_key, &callback_host).await;
            momo_result.expect("Failed to initialize Momo")
        })
        .await
    }

    #[tokio::test]
    async fn test_deposit_successful() {
        let momo = get_momo().await;
        let primary_key = env::var("MTN_DISBURSEMENT_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key =
            env::var("MTN_DISBURSEMENT_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let disbursements = momo.disbursement(primary_key, secondary_key);

        let mut d = CallbackTestHelper::new()
            .await
            .expect("Failed to start callback listener");

        let payee = Party {
            party_id_type: PartyIdType::MSISDN,
            party_id: "22997108557".to_string(), // A number that should succeed
        };
        let transfer = TransferRequest::new(
            "100".to_string(),
            Currency::EUR,
            payee,
            "payer_message".to_string(),
            "payee_note".to_string(),
        );

        let call_back_server_url = env::var("CALLBACK_SERVER_URL")
            .unwrap_or_else(|_| "http://webhook.site/0e1ea918-075d-4916-8bf2-a8b696cf82f4".to_string());

        let result = disbursements
            .deposit_v1(transfer.clone(), Some(&call_back_server_url))
            .await;
        assert!(result.is_ok());

        if let Some(callback) = d.next().await {
            if let mtnmomo::CallbackResponse::DisbursementDepositV1Success {
                financial_transaction_id,
                external_id,
                amount,
                currency,
                payee,
                payee_note,
                payer_message: _,
                status,
            } = callback.response
            {
                assert_eq!(external_id, transfer.external_id);
                assert_eq!(amount, transfer.amount);
                assert_eq!(currency, transfer.currency.to_string());
                assert_eq!(payee.party_id, transfer.payee.party_id);
                assert_eq!(payee_note, Some(transfer.payee_note));
                assert_eq!(status, DisbursementDepositV1SuccessStatus::SUCCESSFUL);
                assert!(!financial_transaction_id.is_empty());
            } else {
                panic!(
                    "Expected DisbursementDepositV1Success callback, got {:?}",
                    callback.response
                );
            }
        } else {
            panic!("Did not receive callback");
        }
    }

    macro_rules! test_deposit_failure {
        ($test_name:ident, $phone_number:expr, $expected_reason:expr) => {
            #[tokio::test]
            async fn $test_name() {
                let momo = get_momo().await;
                let primary_key =
                    env::var("MTN_DISBURSEMENT_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
                let secondary_key =
                    env::var("MTN_DISBURSEMENT_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
                let disbursements = momo.disbursement(primary_key, secondary_key);

                let mut d = CallbackTestHelper::new()
                    .await
                    .expect("Failed to start callback listener");

                let payee = Party {
                    party_id_type: PartyIdType::MSISDN,
                    party_id: $phone_number.to_string(),
                };
                let transfer = TransferRequest::new(
                    "100".to_string(),
                    Currency::EUR,
                    payee,
                    "payer_message".to_string(),
                    "payee_note".to_string(),
                );

                let call_back_server_url = env::var("CALLBACK_SERVER_URL").unwrap_or_else(|_| {
                    "http://webhook.site/0e1ea918-075d-4916-8bf2-a8b696cf82f4".to_string()
                });

                let result = disbursements
                    .deposit_v1(transfer.clone(), Some(&call_back_server_url))
                    .await;
                assert!(result.is_ok());

                if let Some(callback) = d.next().await {
                    if let mtnmomo::CallbackResponse::DisbursementDepositV1Failed {
                        financial_transaction_id: _,
                        external_id,
                        amount,
                        currency,
                        payee,
                        payee_note,
                        payer_message: _, // ignore if not needed
                        status,
                        reason,
                    } = callback.response
                    {
                        assert_eq!(external_id, transfer.external_id);
                        assert_eq!(amount, transfer.amount);
                        assert_eq!(currency, transfer.currency.to_string());
                        assert_eq!(payee.party_id, transfer.payee.party_id);
                        assert_eq!(payee_note, Some(transfer.payee_note));
                        assert_eq!(status, DisbursementDepositV1FailedStatus::FAILED);
                        assert_eq!(reason.code, $expected_reason);
                    } else {
                        panic!(
                            "Expected DisbursementDepositV1Failed callback, got {:?}",
                            callback.response
                        );
                    }
                } else {
                    panic!("Did not receive callback");
                }
            }
        };
    }

    test_deposit_failure!(
        test_deposit_payer_failed,
        "46733123450",
        RequestToPayReason::InternalProcessingError
    );
    test_deposit_failure!(
        test_deposit_payer_rejected,
        "46733123451",
        RequestToPayReason::APPROVALREJECTED
    );
    test_deposit_failure!(
        test_deposit_payer_expired,
        "46733123452",
        RequestToPayReason::EXPIRED
    );
    test_deposit_failure!(
        test_deposit_payer_ongoing,
        "46733123453",
        RequestToPayReason::ONGOING
    );
    test_deposit_failure!(
        test_deposit_payer_delayed,
        "46733123454",
        RequestToPayReason::PAYERDELAYED
    );
    test_deposit_failure!(
        test_deposit_payer_not_found,
        "46733123455",
        RequestToPayReason::PAYERNOTFOUND
    );
    test_deposit_failure!(
        test_deposit_payer_payee_not_allowed_to_receive,
        "46733123456",
        RequestToPayReason::PAYEENOTALLOWEDTORECEIVE
    );
    test_deposit_failure!(
        test_deposit_payer_not_allowed,
        "46733123457",
        RequestToPayReason::NOTALLOWED
    );
    test_deposit_failure!(
        test_deposit_payer_not_allowed_target_environment,
        "46733123458",
        RequestToPayReason::NOTALLOWEDTARGETENVIRONMENT
    );
    test_deposit_failure!(
        test_deposit_payer_invalid_callback_url_host,
        "46733123459",
        RequestToPayReason::INVALIDCALLBACKURLHOST
    );
    test_deposit_failure!(
        test_deposit_payer_invalid_currency,
        "46733123460",
        RequestToPayReason::INVALIDCURRENCY
    );
    test_deposit_failure!(
        test_deposit_payer_internal_processing_error,
        "46733123461",
        RequestToPayReason::InternalProcessingError
    );
    test_deposit_failure!(
        test_deposit_payer_service_unavailable,
        "46733123462",
        RequestToPayReason::SERVICEUNAVAILABLE
    );
    test_deposit_failure!(
        test_deposit_payer_could_not_perform_transaction,
        "46733123463",
        RequestToPayReason::COULDNOTPERFORMTRANSACTION
    );

    macro_rules! test_refund_failure {
        ($test_name:ident, $reference_id_to_refund:expr, $expected_reason:expr) => {
            #[tokio::test]
            async fn $test_name() {
                let momo = get_momo().await;
                let primary_key =
                    env::var("MTN_DISBURSEMENT_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
                let secondary_key =
                    env::var("MTN_DISBURSEMENT_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
                let disbursements = momo.disbursement(primary_key, secondary_key);

                let mut d = CallbackTestHelper::new()
                    .await
                    .expect("Failed to start callback listener");

                let refund = RefundRequest::new(
                    "100".to_string(),
                    Currency::EUR.to_string(),
                    "payer_message".to_string(),
                    "payee_note".to_string(),
                    $reference_id_to_refund.to_string(),
                );

                let call_back_server_url = env::var("CALLBACK_SERVER_URL").unwrap_or_else(|_| {
                    "http://webhook.site/0e1ea918-075d-4916-8bf2-a8b696cf82f4".to_string()
                });

                let refund_res = disbursements
                    .refund_v1(refund.clone(), Some(&call_back_server_url))
                    .await;
                assert!(refund_res.is_ok());

                if let Some(callback) = d.next().await {
                    if let mtnmomo::CallbackResponse::DisbursementRefundV1Failed {
                        financial_transaction_id: _,
                        external_id,
                        amount,
                        currency,
                        payee: _,
                        payee_note,
                        payer_message: _, // ignore if not needed
                        status,
                        reason,
                    } = callback.response
                    {
                        assert_eq!(external_id, refund.external_id);
                        assert_eq!(amount, refund.amount);
                        assert_eq!(currency, refund.currency);
                        assert_eq!(payee_note, Some(refund.payee_note.clone()));
                        assert_eq!(status, DisbursementRefundV1FailedStatus::FAILED);
                        assert_eq!(reason.code, $expected_reason);
                    } else {
                        panic!(
                            "Expected DisbursementRefundV1Failed callback, got {:?}",
                            callback.response
                        );
                    }
                } else {
                    panic!("Did not receive callback");
                }
            }
        };
    }

    test_refund_failure!(
        test_refund_transaction_not_found,
        "1",
        RequestToPayReason::PAYERNOTFOUND
    );
    test_refund_failure!(
        test_refund_transaction_failed,
        "2",
        RequestToPayReason::InternalProcessingError
    );
    test_refund_failure!(
        test_refund_transaction_rejected,
        "3",
        RequestToPayReason::APPROVALREJECTED
    );
    test_refund_failure!(
        test_refund_transaction_expired,
        "4",
        RequestToPayReason::EXPIRED
    );
    test_refund_failure!(
        test_refund_transaction_ongoing,
        "5",
        RequestToPayReason::ONGOING
    );
    test_refund_failure!(
        test_refund_transaction_delayed,
        "6",
        RequestToPayReason::PAYERDELAYED
    );
    test_refund_failure!(
        test_refund_transaction_not_allowed,
        "7",
        RequestToPayReason::NOTALLOWED
    );
    test_refund_failure!(
        test_refund_transaction_not_allowed_target_environment,
        "8",
        RequestToPayReason::NOTALLOWEDTARGETENVIRONMENT
    );
    test_refund_failure!(
        test_refund_transaction_invalid_callback_url_host,
        "9",
        RequestToPayReason::INVALIDCALLBACKURLHOST
    );
    test_refund_failure!(
        test_refund_transaction_invalid_currency,
        "10",
        RequestToPayReason::INVALIDCURRENCY
    );
    test_refund_failure!(
        test_refund_transaction_internal_processing_error,
        "11",
        RequestToPayReason::InternalProcessingError
    );
    test_refund_failure!(
        test_refund_transaction_service_unavailable,
        "12",
        RequestToPayReason::SERVICEUNAVAILABLE
    );
    test_refund_failure!(
        test_refund_transaction_could_not_perform_transaction,
        "13",
        RequestToPayReason::COULDNOTPERFORMTRANSACTION
    );

    #[tokio::test]
    async fn test_transfer_successful() {
        let momo = get_momo().await;
        let primary_key = env::var("MTN_DISBURSEMENT_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key =
            env::var("MTN_DISBURSEMENT_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let disbursements = momo.disbursement(primary_key, secondary_key);

        let mut d = CallbackTestHelper::new()
            .await
            .expect("Failed to start callback listener");

        let payee = Party {
            party_id_type: PartyIdType::MSISDN,
            party_id: "22997108557".to_string(), // A number that should succeed
        };
        let transfer = TransferRequest::new(
            "100".to_string(),
            Currency::EUR,
            payee,
            "payer_message".to_string(),
            "payee_note".to_string(),
        );

        let call_back_server_url = env::var("CALLBACK_SERVER_URL")
            .unwrap_or_else(|_| "http://webhook.site/0e1ea918-075d-4916-8bf2-a8b696cf82f4".to_string());

        let result = disbursements
            .transfer(transfer.clone(), Some(&call_back_server_url))
            .await;
        assert!(result.is_ok());

        if let Some(callback) = d.next().await {
            if let mtnmomo::CallbackResponse::DisbursementTransferSuccess {
                financial_transaction_id,
                external_id,
                amount,
                currency,
                payee,
                payee_note,
                payer_message: _,
                status,
            } = callback.response
            {
                assert_eq!(external_id, transfer.external_id);
                assert_eq!(amount, transfer.amount);
                assert_eq!(currency, transfer.currency.to_string());
                assert_eq!(payee.party_id, transfer.payee.party_id);
                assert_eq!(payee_note, Some(transfer.payee_note));
                assert_eq!(status, DisbursementTransferSuccessStatus::SUCCESSFUL);
                assert!(!financial_transaction_id.is_empty());
            } else {
                panic!(
                    "Expected DisbursementTransferSuccess callback, got {:?}",
                    callback.response
                );
            }
        } else {
            panic!("Did not receive callback");
        }
    }

    macro_rules! test_transfer_failure {
        ($test_name:ident, $phone_number:expr, $expected_reason:expr) => {
            #[tokio::test]
            async fn $test_name() {
                let momo = get_momo().await;
                let primary_key =
                    env::var("MTN_DISBURSEMENT_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
                let secondary_key =
                    env::var("MTN_DISBURSEMENT_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
                let disbursements = momo.disbursement(primary_key, secondary_key);

                let mut d = CallbackTestHelper::new()
                    .await
                    .expect("Failed to start callback listener");

                let payee = Party {
                    party_id_type: PartyIdType::MSISDN,
                    party_id: $phone_number.to_string(),
                };
                let transfer = TransferRequest::new(
                    "100".to_string(),
                    Currency::EUR,
                    payee,
                    "payer_message".to_string(),
                    "payee_note".to_string(),
                );

                let call_back_server_url = env::var("CALLBACK_SERVER_URL").unwrap_or_else(|_| {
                    "http://webhook.site/0e1ea918-075d-4916-8bf2-a8b696cf82f4".to_string()
                });

                let result = disbursements
                    .transfer(transfer.clone(), Some(&call_back_server_url))
                    .await;
                assert!(result.is_ok());

                if let Some(callback) = d.next().await {
                    if let mtnmomo::CallbackResponse::DisbursementTransferFailed {
                        financial_transaction_id: _,
                        external_id,
                        amount,
                        currency,
                        payee,
                        payee_note,
                        payer_message: _, // ignore if not needed
                        status,
                        reason,
                    } = callback.response
                    {
                        assert_eq!(external_id, transfer.external_id);
                        assert_eq!(amount, transfer.amount);
                        assert_eq!(currency, transfer.currency.to_string());
                        assert_eq!(payee.party_id, transfer.payee.party_id);
                        assert_eq!(payee_note, Some(transfer.payee_note));
                        assert_eq!(status, DisbursementTransferFailedStatus::FAILED);
                        assert_eq!(reason.code, $expected_reason);
                    } else {
                        panic!(
                            "Expected DisbursementTransferFailed callback, got {:?}",
                            callback.response
                        );
                    }
                } else {
                    panic!("Did not receive callback");
                }
            }
        };
    }

    test_transfer_failure!(
        test_transfer_payee_failed,
        "46733123450",
        RequestToPayReason::InternalProcessingError
    );
    test_transfer_failure!(
        test_transfer_payee_rejected,
        "46733123451",
        RequestToPayReason::APPROVALREJECTED
    );
    test_transfer_failure!(
        test_transfer_payee_expired,
        "46733123452",
        RequestToPayReason::EXPIRED
    );
    test_transfer_failure!(
        test_transfer_payee_ongoing,
        "46733123453",
        RequestToPayReason::ONGOING
    );
    test_transfer_failure!(
        test_transfer_payee_delayed,
        "46733123454",
        RequestToPayReason::PAYERDELAYED
    );
    test_transfer_failure!(
        test_transfer_payee_not_enough_funds,
        "46733123455",
        RequestToPayReason::COULDNOTPERFORMTRANSACTION
    );
    test_transfer_failure!(
        test_transfer_payee_payer_limit_reached,
        "46733123456",
        RequestToPayReason::COULDNOTPERFORMTRANSACTION
    );
    test_transfer_failure!(
        test_transfer_payee_not_found,
        "46733123457",
        RequestToPayReason::PAYERNOTFOUND
    );
    test_transfer_failure!(
        test_transfer_payee_not_allowed,
        "46733123458",
        RequestToPayReason::NOTALLOWED
    );
    test_transfer_failure!(
        test_transfer_payee_not_allowed_target_environment,
        "46733123459",
        RequestToPayReason::NOTALLOWEDTARGETENVIRONMENT
    );
    test_transfer_failure!(
        test_transfer_payee_invalid_callback_url_host,
        "46733123460",
        RequestToPayReason::INVALIDCALLBACKURLHOST
    );
    test_transfer_failure!(
        test_transfer_payee_invalid_currency,
        "46733123461",
        RequestToPayReason::INVALIDCURRENCY
    );
    test_transfer_failure!(
        test_transfer_payee_internal_processing_error,
        "46733123462",
        RequestToPayReason::InternalProcessingError
    );
    test_transfer_failure!(
        test_transfer_payee_service_unavailable,
        "46733123463",
        RequestToPayReason::SERVICEUNAVAILABLE
    );
}
