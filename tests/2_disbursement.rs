mod common;

#[cfg(test)]
mod tests {
    use crate::common::CallbackTestHelper;
    use futures_util::StreamExt;
    use mtnmomo::{
        callback::{
            DisbursementDepositV1FailedStatus,  DisbursementRefundV1FailedStatus, DisbursementSuccessStatus, DisbursementFailedStatus
        }, enums::reason::RequestToPayReason, Currency, Momo, Party, PartyIdType, RefundRequest, TransferRequest
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

        let mut callback_helper = CallbackTestHelper::new()
            .await
            .expect("Failed to start callback listener");
        let stream_result = callback_helper.listen().await;
        assert!(stream_result.is_ok());
        let mut stream = stream_result.unwrap().boxed();

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

    if let Some(callback) = stream.next().await {
            if let mtnmomo::CallbackResponse::DisbursementSuccess {
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
                assert_eq!(status, DisbursementSuccessStatus::SUCCESSFUL);
                assert!(!financial_transaction_id.is_empty());
            } else {
                panic!(
                    "Expected DisbursementSuccess callback, got {:?}",
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

                let mut callback_helper = CallbackTestHelper::new()
                .await
                .expect("Failed to start callback listener");

                let stream_result = callback_helper.listen().await;
                assert!(stream_result.is_ok());
                let mut stream = stream_result.unwrap().boxed();

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

                if let Some(callback) = stream.next().await {
                    if let mtnmomo::CallbackResponse::DisbursementFailed {
                        external_id,
                        amount,
                        currency,
                        payee,
                        payee_note,
                        status,
                        reason,
                    } = callback.response
                    {
                        assert_eq!(external_id, transfer.external_id);
                        assert_eq!(amount, transfer.amount);
                        assert_eq!(currency, transfer.currency.to_string());
                        assert_eq!(payee.party_id, transfer.payee.party_id);
                        assert_eq!(payee_note, Some(transfer.payee_note));
                        assert_eq!(status, DisbursementFailedStatus::FAILED);
                        assert_eq!(reason, $expected_reason);
                    } else {
                        panic!(
                            "Expected DisbursementFailed callback, got {:?}",
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
        Some(RequestToPayReason::InternalProcessingError)
    );
    test_deposit_failure!(
        test_deposit_payer_rejected,
        "46733123451",
        Some(RequestToPayReason::APPROVALREJECTED)
    );
    test_deposit_failure!(
        test_deposit_payer_expired,
        "46733123452",
        Some(RequestToPayReason::EXPIRED)
    );
    test_deposit_failure!(
        test_deposit_payer_ongoing,
        "46733123453",
        Some(RequestToPayReason::ONGOING)
    );
    test_deposit_failure!(
        test_deposit_payer_delayed,
        "46733123454",
        Some(RequestToPayReason::PAYERDELAYED)
    );
    test_deposit_failure!(
        test_deposit_payer_not_found,
        "46733123455",
        Some(RequestToPayReason::PAYERNOTFOUND)
    );
    test_deposit_failure!(
        test_deposit_payer_payee_not_allowed_to_receive,
        "46733123456",
        Some(RequestToPayReason::PAYEENOTALLOWEDTORECEIVE)
    );
    test_deposit_failure!(
        test_deposit_payer_not_allowed,
        "46733123457",
        Some(RequestToPayReason::NOTALLOWED)
    );
    test_deposit_failure!(
        test_deposit_payer_not_allowed_target_environment,
        "46733123458",
        Some(RequestToPayReason::NOTALLOWEDTARGETENVIRONMENT)
    );
    test_deposit_failure!(
        test_deposit_payer_invalid_callback_url_host,
        "46733123459",
        Some(RequestToPayReason::INVALIDCALLBACKURLHOST)
    );
    test_deposit_failure!(
        test_deposit_payer_invalid_currency,
        "46733123460",
        Some(RequestToPayReason::INVALIDCURRENCY)
    );
    test_deposit_failure!(
        test_deposit_payer_internal_processing_error,
        "46733123461",
        Some(RequestToPayReason::InternalProcessingError)
    );
    test_deposit_failure!(
        test_deposit_payer_service_unavailable,
        "46733123462",
        Some(RequestToPayReason::SERVICEUNAVAILABLE)
    );
    test_deposit_failure!(
        test_deposit_payer_could_not_perform_transaction,
        "46733123463",
        Some(RequestToPayReason::COULDNOTPERFORMTRANSACTION)
    );


    #[tokio::test]
    async fn test_transfer_successful() {
        let momo = get_momo().await;
        let primary_key = env::var("MTN_DISBURSEMENT_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key =
            env::var("MTN_DISBURSEMENT_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let disbursements = momo.disbursement(primary_key, secondary_key);

        let mut callback_helper = CallbackTestHelper::new()
            .await
            .expect("Failed to start callback listener");

        let stream_result = callback_helper.listen().await;
        assert!(stream_result.is_ok());
        let mut stream = stream_result.unwrap().boxed();

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

        if let Some(callback) = stream.next().await {
            if let mtnmomo::CallbackResponse::DisbursementSuccess {
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
                assert_eq!(status, DisbursementSuccessStatus::SUCCESSFUL);
                assert!(!financial_transaction_id.is_empty());
            } else {
                panic!(
                    "Expected DisbursementSuccess callback, got {:?}",
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

                let mut callback_helper = CallbackTestHelper::new()
                    .await
                    .expect("Failed to start callback listener");

                let stream_result = callback_helper.listen().await;
                assert!(stream_result.is_ok());
                let mut stream = stream_result.unwrap().boxed();

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

                if let Some(callback) = stream.next().await {
                    if let mtnmomo::CallbackResponse::DisbursementFailed {
                        external_id,
                        amount,
                        currency,
                        payee,
                        payee_note,
                        status,
                        reason,
                    } = callback.response
                    {
                        assert_eq!(external_id, transfer.external_id);
                        assert_eq!(amount, transfer.amount);
                        assert_eq!(currency, transfer.currency.to_string());
                        assert_eq!(payee.party_id, transfer.payee.party_id);
                        assert_eq!(payee_note, Some(transfer.payee_note));
                        assert_eq!(status, DisbursementFailedStatus::FAILED);
                        assert_eq!(reason, $expected_reason);
                    } else {
                        panic!(
                            "Expected DisbursementFailed callback, got {:?}",
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
        Some(RequestToPayReason::InternalProcessingError)
    );
    test_transfer_failure!(
        test_transfer_payee_rejected,
        "46733123451",
        Some(RequestToPayReason::APPROVALREJECTED)
    );
    test_transfer_failure!(
        test_transfer_payee_expired,
        "46733123452",
        Some(RequestToPayReason::EXPIRED)
    );
    test_transfer_failure!(
        test_transfer_payee_ongoing,
        "46733123453",
        Some(RequestToPayReason::ONGOING)
    );
    test_transfer_failure!(
        test_transfer_payee_delayed,
        "46733123454",
        Some(RequestToPayReason::PAYERDELAYED)
    );
    test_transfer_failure!(
        test_transfer_payee_not_enough_funds,
        "46733123455",
        Some(RequestToPayReason::COULDNOTPERFORMTRANSACTION)
    );
    test_transfer_failure!(
        test_transfer_payee_payer_limit_reached,
        "46733123456",
        Some(RequestToPayReason::COULDNOTPERFORMTRANSACTION)
    );
    test_transfer_failure!(
        test_transfer_payee_not_found,
        "46733123457",
        Some(RequestToPayReason::PAYERNOTFOUND)
    );
    test_transfer_failure!(
        test_transfer_payee_not_allowed,
        "46733123458",
        Some(RequestToPayReason::NOTALLOWED)
    );
    test_transfer_failure!(
        test_transfer_payee_not_allowed_target_environment,
        "46733123459",
        Some(RequestToPayReason::NOTALLOWEDTARGETENVIRONMENT)
    );
    test_transfer_failure!(
        test_transfer_payee_invalid_callback_url_host,
        "46733123460",
        Some(RequestToPayReason::INVALIDCALLBACKURLHOST)
    );
    test_transfer_failure!(
        test_transfer_payee_invalid_currency,
        "46733123461",
        Some(RequestToPayReason::INVALIDCURRENCY)
    );
    test_transfer_failure!(
        test_transfer_payee_internal_processing_error,
        "46733123462",
        Some(RequestToPayReason::InternalProcessingError)
    );
    test_transfer_failure!(
        test_transfer_payee_service_unavailable,
        "46733123463",
        Some(RequestToPayReason::SERVICEUNAVAILABLE)
    );
}
