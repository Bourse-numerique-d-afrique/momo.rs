mod common;

#[cfg(test)]
mod tests {
    use crate::common::CallbackTestHelper;
    use futures_util::StreamExt;
    use mtnmomo::{
        enums::{payer_identification_type::PayerIdentificationType, reason::RequestToPayReason},
        CashTransferRequest, Currency, Momo, Party, PartyIdType, TransferRequest,
    };
    use std::env;
    use tokio::sync::OnceCell;

    static MOMO: OnceCell<Momo> = OnceCell::const_new();

    async fn get_momo() -> &'static Momo {
        MOMO.get_or_init(|| async {
            let mtn_url = env::var("MTN_URL").expect("MTN_URL must be set");
            let subscription_key =
                env::var("MTN_REMITTANCE_PRIMARY_KEY").expect("PRIMARY_KEY must be set");

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
    async fn test_cash_transfer_successful() {
        let momo = get_momo().await;
        let primary_key = env::var("MTN_REMITTANCE_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key =
            env::var("MTN_REMITTANCE_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let remittance = momo.remittance(primary_key, secondary_key);

        let mut callback_helper = CallbackTestHelper::new()
            .await
            .expect("Failed to start callback listener");

        let stream_result = callback_helper.listen().await;
        assert!(stream_result.is_ok());
        let mut stream = stream_result.unwrap().boxed();

        let payee = Party {
            party_id_type: PartyIdType::MSISDN,
            party_id: "22997108557".to_string(),
        };
        let request = CashTransferRequest::new(
            "1000".to_string(),
            Currency::EUR,
            payee,
            "UG".to_string(),
            "1000".to_string(),
            Currency::EUR,
            "payer_message".to_string(),
            "payee_note".to_string(),
            PayerIdentificationType::PASS,
            "256774290781".to_string(),
            "256774290781".to_string(),
            "John".to_string(),
            "Doe".to_string(),
            "en".to_string(),
            "test@email.com".to_string(),
            "256774290781".to_string(),
            "M".to_string(),
        );

        let call_back_server_url = env::var("CALLBACK_SERVER_URL")
            .unwrap_or_else(|_| "http://webhook.site/0e1ea918-075d-4916-8bf2-a8b696cf82f4".to_string());

        let res = remittance
            .cash_transfer(request.clone(), Some(&call_back_server_url))
            .await;
        assert!(res.is_ok());

    if let Some(callback) = stream.next().await {
            if let mtnmomo::CallbackResponse::CashTransferSucceeded { external_id, .. } =
                callback.response
            {
                assert_eq!(external_id, request.external_id);
            } else {
                panic!(
                    "Expected CashTransferSucceeded callback, got {:?}",
                    callback.response
                );
            }
        } else {
            panic!("Did not receive callback");
        }
    }

    macro_rules! test_cash_transfer_failure {
        ($test_name:ident, $payer_id:expr, $expected_reason:expr) => {
            #[tokio::test]
            async fn $test_name() {
                let momo = get_momo().await;
                let primary_key =
                    env::var("MTN_REMITTANCE_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
                let secondary_key =
                    env::var("MTN_REMITTANCE_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
                let remittance = momo.remittance(primary_key, secondary_key);

                let mut d = CallbackTestHelper::new()
                    .await
                    .expect("Failed to start callback listener");
                let stream_result = d.listen().await;
                assert!(stream_result.is_ok());
                let mut stream = stream_result.unwrap().boxed();

                let payee = Party {
                    party_id_type: PartyIdType::MSISDN,
                    party_id: "22997108557".to_string(),
                };

                let request = CashTransferRequest::new(
                    "100".to_string(),
                    Currency::EUR,
                    payee,
                    "UG".to_string(),
                    "100".to_string(),
                    Currency::EUR,
                    "payer_message".to_string(),
                    "payee_note".to_string(),
                    PayerIdentificationType::PASS,
                    $payer_id.to_string(),
                    "256774290781".to_string(),
                    "John".to_string(),
                    "Doe".to_string(),
                    "en".to_string(),
                    "test@email.com".to_string(),
                    "256774290781".to_string(),
                    "M".to_string(),
                );

                let call_back_server_url = env::var("CALLBACK_SERVER_URL").unwrap_or_else(|_| {
                    "http://webhook.site/0e1ea918-075d-4916-8bf2-a8b696cf82f4".to_string()
                });

                let res = remittance
                    .cash_transfer(request.clone(), Some(&call_back_server_url))
                    .await;
                assert!(res.is_ok());

                if let Some(callback) = stream.next().await {
                    if let mtnmomo::CallbackResponse::CashTransferFailed {
                        error_reason,
                        external_id,
                        .. 
                    } = callback.response
                    {
                        assert_eq!(external_id, request.external_id);
                        assert_eq!(error_reason.code, $expected_reason);
                    } else {
                        panic!(
                            "Expected CashTransferFailed callback, got {:?}",
                            callback.response
                        );
                    }
                } else {
                    panic!("Did not receive callback");
                }
            }
        };
    }

    test_cash_transfer_failure!(
        test_cash_transfer_transaction_not_found,
        "1",
        RequestToPayReason::PAYERNOTFOUND
    );
    test_cash_transfer_failure!(
        test_cash_transfer_transaction_failed,
        "2",
        RequestToPayReason::InternalProcessingError
    );
    test_cash_transfer_failure!(
        test_cash_transfer_transaction_rejected,
        "3",
        RequestToPayReason::APPROVALREJECTED
    );
    test_cash_transfer_failure!(
        test_cash_transfer_transaction_expired,
        "4",
        RequestToPayReason::EXPIRED
    );
    test_cash_transfer_failure!(
        test_cash_transfer_transaction_payee_not_found,
        "5",
        RequestToPayReason::PAYERNOTFOUND
    );
    test_cash_transfer_failure!(
        test_cash_transfer_transaction_payee_not_allowed_to_receive,
        "6",
        RequestToPayReason::PAYEENOTALLOWEDTORECEIVE
    );
    test_cash_transfer_failure!(
        test_cash_transfer_transaction_not_allowed,
        "7",
        RequestToPayReason::NOTALLOWED
    );
    test_cash_transfer_failure!(
        test_cash_transfer_transaction_not_allowed_target_environment,
        "8",
        RequestToPayReason::NOTALLOWEDTARGETENVIRONMENT
    );
    test_cash_transfer_failure!(
        test_cash_transfer_transaction_invalid_callback_url_host,
        "9",
        RequestToPayReason::INVALIDCALLBACKURLHOST
    );
    test_cash_transfer_failure!(
        test_cash_transfer_transaction_invalid_currency,
        "10",
        RequestToPayReason::INVALIDCURRENCY
    );
    test_cash_transfer_failure!(
        test_cash_transfer_transaction_internal_processing_error,
        "11",
        RequestToPayReason::InternalProcessingError
    );
    test_cash_transfer_failure!(
        test_cash_transfer_transaction_service_unavailable,
        "12",
        RequestToPayReason::SERVICEUNAVAILABLE
    );
    test_cash_transfer_failure!(
        test_cash_transfer_transaction_could_not_perform_transaction,
        "13",
        RequestToPayReason::COULDNOTPERFORMTRANSACTION
    );

    #[tokio::test]
    async fn test_transfer_successful() {
        let momo = get_momo().await;
        let primary_key = env::var("MTN_REMITTANCE_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key =
            env::var("MTN_REMITTANCE_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let remittance = momo.remittance(primary_key, secondary_key);

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

        let result = remittance
            .transfer(transfer.clone(), Some(&call_back_server_url))
            .await;
        assert!(result.is_ok());

    if let Some(callback) = stream.next().await {
            if let mtnmomo::CallbackResponse::RemittanceTransferSuccess { external_id, .. } =
                callback.response
            {
                assert_eq!(external_id, transfer.external_id);
            } else {
                panic!(
                    "Expected RemittanceTransferSuccess callback, got {:?}",
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
                    env::var("MTN_REMITTANCE_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
                let secondary_key =
                    env::var("MTN_REMITTANCE_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
                let remittance = momo.remittance(primary_key, secondary_key);

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

                let result = remittance
                    .transfer(transfer.clone(), Some(&call_back_server_url))
                    .await;
                assert!(result.is_ok());

                if let Some(callback) = stream.next().await {
                    if let mtnmomo::CallbackResponse::RemittanceTransferFailed {
                        error_reason,
                        external_id,
                        .. 
                    } = callback.response
                    {
                        assert_eq!(external_id, transfer.external_id);
                        assert_eq!(error_reason, $expected_reason);
                    } else {
                        panic!(
                            "Expected RemittanceTransferFailed callback, got {:?}",
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
