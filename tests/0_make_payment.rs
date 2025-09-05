mod common;

#[cfg(all(test, not(feature = "skip-integration-tests")))]
mod tests {
    use crate::common::CallbackTestHelper;
    use futures_core::Stream;
    use futures_util::StreamExt;
    use mtnmomo::{
        callback::{
            PreApprovalCreatedStatus, PreApprovalSuccessfulStatus, RequestToPayFailedStatus,
            RequestToPaySuccessfulStatus,
        },
        enums::reason::RequestToPayReason,
        Currency, Momo, MomoUpdates, Party, PartyIdType, PreApprovalRequest, RequestToPay,
    };
    use std::{env, time::Duration};
    use tokio::{sync::OnceCell, time::timeout};

    static MOMO: OnceCell<Momo> = OnceCell::const_new();

    // Helper function to drain all pending callbacks from a stream
    async fn drain_stream<S>(stream: &mut S) -> usize
    where
        S: Stream<Item = MomoUpdates> + Unpin,
    {
        let mut drained_count = 0;
        let drain_timeout = Duration::from_millis(100); // Short timeout for draining

        while let Ok(Some(_)) = timeout(drain_timeout, stream.next()).await {
            drained_count += 1;
            println!("🧹 Drained stale callback #{}", drained_count);

            // Safety check to prevent infinite loops
            if drained_count > 50 {
                println!("⚠️  Stopped draining after 50 callbacks to prevent infinite loop");
                break;
            }
        }

        if drained_count > 0 {
            println!("🧹 Total drained: {} stale callbacks", drained_count);
        } else {
            println!("✅ Stream was already clean");
        }

        drained_count
    }

    async fn get_momo() -> &'static Momo {
        MOMO.get_or_init(|| async {
            let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");
            let subscription_key =
                env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");

            let callback_host =
                env::var("MTN_CALLBACK_HOST").unwrap_or_else(|_| "webhook.site".to_string());

            let momo_result =
                Momo::new_with_provisioning(mtn_url, subscription_key, &callback_host).await;
            momo_result.expect("Failed to initialize Momo")
        })
        .await
    }

    #[tokio::test]
    async fn test_0_make_provisioning() {
        let _momo = get_momo().await;
        // Test passes if initialization was successful
    }

    #[tokio::test]
    async fn test_1_make_payment_successful() {
        let momo = get_momo().await;
        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key =
            env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let collection = momo.collection(primary_key, secondary_key);

        let payer: Party = Party {
            party_id_type: PartyIdType::MSISDN,
            party_id: "22997108557".to_string(),
        };

        let mut callback_helper = CallbackTestHelper::new()
            .await
            .expect("Failed to start callback listener");

        let stream_result = callback_helper.listen().await;
        assert!(stream_result.is_ok());
        let mut stream = stream_result.unwrap().boxed();

        let request = RequestToPay::new(
            "100".to_string(),
            Currency::EUR,
            payer,
            "test_payer_message".to_string(),
            "test_payee_note".to_string(),
        );

        let callback_url = env::var("CALLBACK_SERVER_URL").unwrap_or_else(|_| {
            "http://webhook.site/0e1ea918-075d-4916-8bf2-a8b696cf82f4".to_string()
        });

        let request_to_pay_result = collection
            .request_to_pay(request.clone(), Some(&callback_url))
            .await;

        assert!(request_to_pay_result.is_ok());

        if let Some(callback) = stream.next().await {
            if let mtnmomo::CallbackResponse::RequestToPaySuccess {
                financial_transaction_id,
                external_id,
                amount,
                currency,
                payer,
                payee_note,
                payer_message: _, // ignore if not needed
                status,
            } = callback.response
            {
                assert_eq!(external_id, request.external_id);
                assert_eq!(amount, request.amount);
                assert_eq!(currency, request.currency.to_string());
                assert_eq!(payer.party_id, request.payer.party_id);
                assert_eq!(payee_note, Some(request.payee_note));
                assert_eq!(status, RequestToPaySuccessfulStatus::SUCCESSFUL);
                assert!(!financial_transaction_id.is_empty());
            } else {
                panic!(
                    "Expected RequestToPaySuccess callback, got {:?}",
                    callback.response
                );
            }
        } else {
            panic!("Did not receive callback");
        }
    }

    macro_rules! test_request_to_pay_failure {
        ($test_name:ident, $phone_number:expr, $expected_reason:expr) => {
            #[tokio::test]
            async fn $test_name() {
                let momo = get_momo().await;
                let primary_key =
                    env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
                let secondary_key =
                    env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
                let collection = momo.collection(primary_key, secondary_key);

                let mut callback_helper = CallbackTestHelper::new()
                    .await
                    .expect("Failed to start callback listener");

                let stream_result = callback_helper.listen().await;
                assert!(stream_result.is_ok());
                let mut stream = stream_result.unwrap().boxed();

                let payer: Party = Party {
                    party_id_type: PartyIdType::MSISDN,
                    party_id: $phone_number.to_string(),
                };

                let request = RequestToPay::new(
                    "100".to_string(),
                    Currency::EUR,
                    payer,
                    "test_payer_message".to_string(),
                    "test_payee_note".to_string(),
                );

                let call_back_server_url = env::var("CALLBACK_SERVER_URL").unwrap_or_else(|_| {
                    "http://webhook.site/0e1ea918-075d-4916-8bf2-a8b696cf82f4".to_string()
                });

                let request_to_pay_result = collection
                    .request_to_pay(request.clone(), Some(&call_back_server_url))
                    .await;

                assert!(request_to_pay_result.is_ok());

                if let Some(callback) = stream.next().await {
                    if let mtnmomo::CallbackResponse::RequestToPayFailed {
                        financial_transaction_id: _,
                        external_id,
                        amount,
                        currency,
                        payer,
                        payee_note,
                        payer_message: _, // ignore if not needed
                        status,
                        reason,
                    } = callback.response
                    {
                        assert_eq!(payer.party_id, request.payer.party_id);
                        assert_eq!(external_id, request.external_id);
                        assert_eq!(amount, request.amount);
                        assert_eq!(currency, request.currency.to_string());
                        assert_eq!(payee_note, Some(request.payee_note));
                        assert_eq!(status, RequestToPayFailedStatus::FAILED);
                        assert_eq!(reason, $expected_reason);
                    } else {
                        panic!(
                            "Expected RequestToPayFailed callback, got {:?}",
                            callback.response
                        );
                    }
                } else {
                    panic!("Did not receive callback");
                }
            }
        };
    }

    test_request_to_pay_failure!(
        test_request_to_pay_payer_failed,
        "46733123450",
        RequestToPayReason::InternalProcessingError
    );
    test_request_to_pay_failure!(
        test_request_to_pay_payer_rejected,
        "46733123451",
        RequestToPayReason::APPROVALREJECTED
    );
    test_request_to_pay_failure!(
        test_request_to_pay_payer_expired,
        "46733123452",
        RequestToPayReason::EXPIRED
    );
    test_request_to_pay_failure!(
        test_request_to_pay_payer_ongoing,
        "46733123453",
        RequestToPayReason::ONGOING
    );
    test_request_to_pay_failure!(
        test_request_to_pay_payer_delayed,
        "46733123454",
        RequestToPayReason::PAYERDELAYED
    );
    test_request_to_pay_failure!(
        test_request_to_pay_payer_not_found,
        "46733123455",
        RequestToPayReason::PAYERNOTFOUND
    );
    test_request_to_pay_failure!(
        test_request_to_pay_payer_payee_not_allowed_to_receive,
        "46733123456",
        RequestToPayReason::PAYEENOTALLOWEDTORECEIVE
    );
    test_request_to_pay_failure!(
        test_request_to_pay_payer_not_allowed,
        "46733123457",
        RequestToPayReason::NOTALLOWED
    );
    test_request_to_pay_failure!(
        test_request_to_pay_payer_not_allowed_target_environment,
        "46733123458",
        RequestToPayReason::NOTALLOWEDTARGETENVIRONMENT
    );
    test_request_to_pay_failure!(
        test_request_to_pay_payer_invalid_callback_url_host,
        "46733123459",
        RequestToPayReason::INVALIDCALLBACKURLHOST
    );
    test_request_to_pay_failure!(
        test_request_to_pay_payer_invalid_currency,
        "46733123460",
        RequestToPayReason::INVALIDCURRENCY
    );
    test_request_to_pay_failure!(
        test_request_to_pay_payer_internal_processing_error,
        "46733123461",
        RequestToPayReason::InternalProcessingError
    );
    test_request_to_pay_failure!(
        test_request_to_pay_payer_service_unavailable,
        "46733123462",
        RequestToPayReason::SERVICEUNAVAILABLE
    );
    test_request_to_pay_failure!(
        test_request_to_pay_payer_could_not_perform_transaction,
        "46733123463",
        RequestToPayReason::COULDNOTPERFORMTRANSACTION
    );

    #[tokio::test]
    async fn test_request_to_withdraw_successful() {
        let momo = get_momo().await;
        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key =
            env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let collection = momo.collection(primary_key, secondary_key);

        let mut callback_helper = CallbackTestHelper::new()
            .await
            .expect("Failed to start callback listener");

        let payer: Party = Party {
            party_id_type: PartyIdType::MSISDN,
            party_id: "22997108557".to_string(),
        };
        let request = RequestToPay::new(
            "100.0".to_string(),
            Currency::EUR,
            payer,
            "test_payer_message".to_string(),
            "test_payee_note".to_string(),
        );

        let call_back_server_url = env::var("CALLBACK_SERVER_URL").unwrap_or_else(|_| {
            "http://webhook.site/0e1ea918-075d-4916-8bf2-a8b696cf82f4".to_string()
        });

        let stream_result = callback_helper.listen().await;
        assert!(stream_result.is_ok());
        let mut stream = stream_result.unwrap().boxed();

        let res = collection
            .request_to_withdraw_v1(request.clone(), Some(&call_back_server_url))
            .await
            .expect("Error requesting to withdraw");
        assert_ne!(res.as_str().len(), 0);
        if let Some(callback) = stream.next().await {
            if let mtnmomo::CallbackResponse::RequestToPaySuccess {
                financial_transaction_id,
                external_id,
                amount,
                currency,
                payer,
                payee_note,
                payer_message: _, // ignore if not needed
                status,
            } = callback.response
            {
                assert_eq!(external_id, request.external_id);
                assert_eq!(amount, request.amount);
                assert_eq!(currency, request.currency.to_string());
                assert_eq!(payer.party_id, request.payer.party_id);
                assert_eq!(payee_note, Some(request.payee_note));
                assert_eq!(status, RequestToPaySuccessfulStatus::SUCCESSFUL);
                assert!(!financial_transaction_id.is_empty());
            } else {
                panic!(
                    "Expected RequestToPaySuccess callback, got {:?}",
                    callback.response
                );
            }
        } else {
            panic!("Did not receive callback");
        }
    }

    macro_rules! test_request_to_withdraw_failure {
        ($test_name:ident, $phone_number:expr) => {
            #[tokio::test]
            async fn $test_name() {
                let momo = get_momo().await;
                let primary_key =
                    env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
                let secondary_key =
                    env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
                let collection = momo.collection(primary_key, secondary_key);

                let mut callback_helper = CallbackTestHelper::new()
                    .await
                    .expect("Failed to start callback listener");

                let payer: Party = Party {
                    party_id_type: PartyIdType::MSISDN,
                    party_id: $phone_number.to_string(),
                };

                let request = RequestToPay::new(
                    "100".to_string(),
                    Currency::EUR,
                    payer,
                    "test_payer_message".to_string(),
                    "test_payee_note".to_string(),
                );

                let call_back_server_url = env::var("CALLBACK_SERVER_URL").unwrap_or_else(|_| {
                    "http://webhook.site/0e1ea918-075d-4916-8bf2-a8b696cf82f4".to_string()
                });

                let stream_result = callback_helper.listen().await;
                assert!(stream_result.is_ok());
                let mut stream = stream_result.unwrap().boxed();

                let request_to_withdraw_result = collection
                    .request_to_withdraw_v1(request.clone(), Some(&call_back_server_url))
                    .await;

                assert!(request_to_withdraw_result.is_ok());

                if let Some(callback) = stream.next().await {
                    if let mtnmomo::CallbackResponse::RequestToPayFailed {
                        financial_transaction_id: _,
                        external_id,
                        amount,
                        currency,
                        payer,
                        payee_note,
                        payer_message: _, // ignore if not needed
                        status,
                        reason: _, // ignore if not needed
                    } = callback.response
                    {
                        assert_eq!(payer.party_id, request.payer.party_id);
                        assert_eq!(external_id, request.external_id);
                        assert_eq!(amount, request.amount);
                        assert_eq!(currency, request.currency.to_string());
                        assert_eq!(payee_note, Some(request.payee_note));
                        assert_eq!(status, RequestToPayFailedStatus::FAILED);
                    } else {
                        panic!(
                            "Expected RequestToPayFailed callback, got {:?}",
                            callback.response
                        );
                    }
                } else {
                    panic!("Did not receive callback");
                }
            }
        };
    }

    test_request_to_withdraw_failure!(test_request_to_withdraw_payer_failed, "46733123450");

    #[tokio::test]
    async fn test_pre_approval_successful() {
        let momo = get_momo().await;
        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key =
            env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let collection = momo.collection(primary_key, secondary_key);

        let mut callback_helper = CallbackTestHelper::new()
            .await
            .expect("Failed to start callback listener");

        let stream_result = callback_helper.listen().await;
        assert!(stream_result.is_ok());
        let mut stream = stream_result.unwrap().boxed();

        let user: Party = Party {
            party_id_type: PartyIdType::MSISDN,
            party_id: "22997108557".to_string(),
        };
        let preapproval = PreApprovalRequest {
            payer: user,
            payer_currency: Currency::EUR.to_string(),
            payer_message: "".to_string(),
            validity_time: 3600,
        };

        let call_back_server_url = env::var("CALLBACK_SERVER_URL").unwrap_or_else(|_| {
            "http://webhook.site/0e1ea918-075d-4916-8bf2-a8b696cf82f4".to_string()
        });

        let res = collection
            .pre_approval(preapproval.clone(), Some(&call_back_server_url))
            .await;
        assert!(res.is_ok());

        if let Some(callback) = stream.next().await {
            if let mtnmomo::CallbackResponse::PreApprovalSuccess {
                payer,
                payer_currency,
                status,
                expiration_date_time,
            } = callback.response
            {
                assert_eq!(payer.party_id, "22997108557");
                assert_eq!(payer_currency, "EUR");
                assert_eq!(status, PreApprovalSuccessfulStatus::SUCCESSFUL);
                assert!(!expiration_date_time.is_empty());
            } else {
                panic!(
                    "Expected PreApprovalSuccess callback, got {:?}",
                    callback.response
                );
            }
        } else {
            panic!("Did not receive callback");
        }
    }

    // Test for PreApprovalCreated state (ongoing/delayed)
    macro_rules! test_pre_approval_created {
        ($test_name:ident, $phone_number:expr) => {
            #[tokio::test]
            async fn $test_name() {
                let momo = get_momo().await;
                let primary_key =
                    env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
                let secondary_key =
                    env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
                let collection = momo.collection(primary_key, secondary_key);

                let mut callback_helper = CallbackTestHelper::new()
                    .await
                    .expect("Failed to start callback listener");

                let stream_result = callback_helper.listen().await;
                assert!(stream_result.is_ok());
                let mut stream = stream_result.unwrap().boxed();

                // Clear any previous callbacks before waiting for our specific callback

                let user: Party = Party {
                    party_id_type: PartyIdType::MSISDN,
                    party_id: $phone_number.to_string(),
                };
                let preapproval = PreApprovalRequest {
                    payer: user,
                    payer_currency: Currency::EUR.to_string(),
                    payer_message: "".to_string(),
                    validity_time: 3600,
                };

                let call_back_server_url = env::var("CALLBACK_SERVER_URL").unwrap_or_else(|_| {
                    "http://webhook.site/0e1ea918-075d-4916-8bf2-a8b696cf82f4".to_string()
                });

                let res = collection
                    .pre_approval(preapproval.clone(), Some(&call_back_server_url))
                    .await;
                assert!(res.is_ok());

                if let Some(callback) = stream.next().await {
                    if let mtnmomo::CallbackResponse::PreApprovalCreated {
                        payer,
                        payer_currency,
                        status,
                        expiration_date_time,
                    } = callback.response
                    {
                        assert_eq!(payer.party_id, $phone_number);
                        assert_eq!(payer_currency, "EUR");
                        assert_eq!(status, PreApprovalCreatedStatus::CREATED);
                        assert!(!expiration_date_time.is_empty());
                    } else {
                        panic!(
                            "Expected PreApprovalCreated callback, got {:?}",
                            callback.response
                        );
                    }
                } else {
                    panic!("Did not receive callback");
                }
            }
        };
    }

    test_pre_approval_created!(test_pre_approval_payer_ongoing, "46733123453");
    test_pre_approval_created!(test_pre_approval_payer_delayed, "46733123454");

    // Test for PreApprovalFailed state
    macro_rules! test_pre_approval_failure {
        ($test_name:ident, $phone_number:expr, $expected_reason:expr) => {
            #[tokio::test]
            async fn $test_name() {
                let momo = get_momo().await;
                let primary_key =
                    env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
                let secondary_key =
                    env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
                let collection = momo.collection(primary_key, secondary_key);

                let mut callback_helper = CallbackTestHelper::new()
                    .await
                    .expect("Failed to start callback listener");

                let stream_result = callback_helper.listen().await;
                assert!(stream_result.is_ok());
                let mut stream = stream_result.unwrap().boxed();

                // Clear any previous callbacks before waiting for our specific callback

                let user: Party = Party {
                    party_id_type: PartyIdType::MSISDN,
                    party_id: $phone_number.to_string(),
                };
                let preapproval = PreApprovalRequest {
                    payer: user,
                    payer_currency: Currency::EUR.to_string(),
                    payer_message: "".to_string(),
                    validity_time: 3600,
                };

                let call_back_server_url = env::var("CALLBACK_SERVER_URL").unwrap_or_else(|_| {
                    "http://webhook.site/0e1ea918-075d-4916-8bf2-a8b696cf82f4".to_string()
                });

                let res = collection
                    .pre_approval(preapproval.clone(), Some(&call_back_server_url))
                    .await;
                assert!(res.is_ok());

                if let Some(callback) = stream.next().await {
                    if let mtnmomo::CallbackResponse::PreApprovalFailed {
                        payer,
                        payer_currency,
                        expiration_date_time,
                        status: _,
                        reason,
                    } = callback.response
                    {
                        assert_eq!(payer.party_id, $phone_number);
                        assert_eq!(payer_currency, "EUR");
                        assert!(!expiration_date_time.is_empty());

                        // Handle optional expected reason
                        match $expected_reason {
                            Some(expected_reason) => {
                                if let Some(actual_reason) = reason {
                                    assert_eq!(actual_reason, expected_reason);
                                } else {
                                    panic!(
                                        "Expected failure reason {:?}, but got None",
                                        expected_reason
                                    );
                                }
                            }
                            None => {
                                if reason.is_some() {
                                    panic!("Expected no failure reason, but got {:?}", reason);
                                }
                                // else: expected None and got None, which is correct
                            }
                        }
                    } else {
                        panic!(
                            "Expected PreApprovalFailed callback, got {:?}",
                            callback.response
                        );
                    }
                } else {
                    panic!("Did not receive callback");
                }
            }
        };
    }

    test_pre_approval_failure!(test_pre_approval_payer_failed, "46733123450", None);
    test_pre_approval_failure!(
        test_pre_approval_payer_rejected,
        "46733123451",
        Some(RequestToPayReason::APPROVALREJECTED)
    );
    test_pre_approval_failure!(
        test_pre_approval_payer_expired,
        "46733123452",
        Some(RequestToPayReason::EXPIRED)
    );
    test_pre_approval_failure!(
        test_pre_approval_payer_not_found,
        "46733123455",
        Some(RequestToPayReason::PAYERNOTFOUND)
    );
    test_pre_approval_failure!(
        test_pre_approval_payer_not_allowed,
        "46733123456",
        Some(RequestToPayReason::NOTALLOWED)
    );
    test_pre_approval_failure!(
        test_pre_approval_payer_not_allowed_target_environment,
        "46733123457",
        Some(RequestToPayReason::NOTALLOWEDTARGETENVIRONMENT)
    );
    test_pre_approval_failure!(
        test_pre_approval_payer_invalid_callback_url_host,
        "46733123458",
        Some(RequestToPayReason::INVALIDCALLBACKURLHOST)
    );
    test_pre_approval_failure!(
        test_pre_approval_payer_invalid_currency,
        "46733123459",
        Some(RequestToPayReason::INVALIDCURRENCY)
    );
    test_pre_approval_failure!(
        test_pre_approval_payer_internal_processing_error,
        "46733123460",
        Some(RequestToPayReason::InternalProcessingError)
    );
    test_pre_approval_failure!(
        test_pre_approval_payer_service_unavailable,
        "46733123461",
        Some(RequestToPayReason::SERVICEUNAVAILABLE)
    );
}
