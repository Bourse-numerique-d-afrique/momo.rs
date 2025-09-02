mod common;

#[cfg(test)]
mod tests {
    use crate::common::CallbackTestHelper;
    use futures_util::StreamExt;
    use mtnmomo::{
        enums::{reason::RequestToPayReason, request_to_pay_status::RequestToPayStatus},
        Currency, Momo, Party, PartyIdType, PreApprovalRequest, RequestToPay,
    };
    use std::env;
    use tokio::sync::OnceCell;

    static MOMO: OnceCell<Momo> = OnceCell::const_new();

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

        let mut d = CallbackTestHelper::new()
            .await
            .expect("Failed to start callback listener");

        let request = RequestToPay::new(
            "100".to_string(),
            Currency::EUR,
            payer,
            "test_payer_message".to_string(),
            "test_payee_note".to_string(),
        );

        let callback_url = env::var("CALLBACK_SERVER_URL")
            .unwrap_or_else(|_| "http://webhook.site/0e1ea918-075d-4916-8bf2-a8b696cf82f4".to_string());

        let request_to_pay_result = collection
            .request_to_pay(request.clone(), Some(&callback_url))
            .await;

        assert!(request_to_pay_result.is_ok());

        if let Some(callback) = d.next().await {
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
                assert_eq!(status, RequestToPayStatus::SUCCESSFUL);
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

                let mut d = CallbackTestHelper::new()
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

                let request_to_pay_result = collection
                    .request_to_pay(request.clone(), Some(&call_back_server_url))
                    .await;

                assert!(request_to_pay_result.is_ok());

                if let Some(callback) = d.next().await {
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
                        assert_eq!(external_id, request.external_id);
                        assert_eq!(amount, request.amount);
                        assert_eq!(currency, request.currency.to_string());
                        assert_eq!(payer.party_id, request.payer.party_id);
                        assert_eq!(payee_note, Some(request.payee_note));
                        assert_eq!(status, RequestToPayStatus::FAILED);
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

        let call_back_server_url = env::var("CALLBACK_SERVER_URL")
            .unwrap_or_else(|_| "http://webhook.site/0e1ea918-075d-4916-8bf2-a8b696cf82f4".to_string());

        let res = collection
            .request_to_withdraw_v1(request.clone(), Some(&call_back_server_url))
            .await
            .expect("Error requesting to withdraw");
        assert_ne!(res.as_str().len(), 0);
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

                let request_to_withdraw_result = collection
                    .request_to_withdraw_v1(request.clone(), Some(&call_back_server_url))
                    .await;

                assert!(request_to_withdraw_result.is_ok());
            }
        };
    }

    test_request_to_withdraw_failure!(
        test_request_to_withdraw_payer_failed,
        "46733123450"
    );

    #[tokio::test]
    async fn test_pre_approval_successful() {
        let momo = get_momo().await;
        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key =
            env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let collection = momo.collection(primary_key, secondary_key);

        let mut d = CallbackTestHelper::new()
            .await
            .expect("Failed to start callback listener");

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

        let call_back_server_url = env::var("CALLBACK_SERVER_URL")
            .unwrap_or_else(|_| "http://webhook.site/0e1ea918-075d-4916-8bf2-a8b696cf82f4".to_string());

        let res = collection
            .pre_approval(preapproval.clone(), Some(&call_back_server_url))
            .await;
        assert!(res.is_ok());

        if let Some(callback) = d.next().await {
            if let mtnmomo::CallbackResponse::PreApprovalSuccess { .. } = callback.response {
                // success
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

                let mut d = CallbackTestHelper::new()
                    .await
                    .expect("Failed to start callback listener");

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

                if let Some(callback) = d.next().await {
                    if let mtnmomo::CallbackResponse::PreApprovalFailed { reason, .. } =
                        callback.response
                    {
                        assert_eq!(reason.code, $expected_reason);
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

    test_pre_approval_failure!(
        test_pre_approval_payer_failed,
        "46733123450",
        RequestToPayReason::InternalProcessingError
    );
    test_pre_approval_failure!(
        test_pre_approval_payer_rejected,
        "46733123451",
        RequestToPayReason::APPROVALREJECTED
    );
    test_pre_approval_failure!(
        test_pre_approval_payer_expired,
        "46733123452",
        RequestToPayReason::EXPIRED
    );
    test_pre_approval_failure!(
        test_pre_approval_payer_ongoing,
        "46733123453",
        RequestToPayReason::ONGOING
    );
    test_pre_approval_failure!(
        test_pre_approval_payer_delayed,
        "46733123454",
        RequestToPayReason::PAYERDELAYED
    );
    test_pre_approval_failure!(
        test_pre_approval_payer_not_found,
        "46733123455",
        RequestToPayReason::PAYERNOTFOUND
    );
    test_pre_approval_failure!(
        test_pre_approval_payer_not_allowed,
        "46733123456",
        RequestToPayReason::NOTALLOWED
    );
    test_pre_approval_failure!(
        test_pre_approval_payer_not_allowed_target_environment,
        "46733123457",
        RequestToPayReason::NOTALLOWEDTARGETENVIRONMENT
    );
    test_pre_approval_failure!(
        test_pre_approval_payer_invalid_callback_url_host,
        "46733123458",
        RequestToPayReason::INVALIDCALLBACKURLHOST
    );
    test_pre_approval_failure!(
        test_pre_approval_payer_invalid_currency,
        "46733123459",
        RequestToPayReason::INVALIDCURRENCY
    );
    test_pre_approval_failure!(
        test_pre_approval_payer_internal_processing_error,
        "46733123460",
        RequestToPayReason::InternalProcessingError
    );
    test_pre_approval_failure!(
        test_pre_approval_payer_service_unavailable,
        "46733123461",
        RequestToPayReason::SERVICEUNAVAILABLE
    );
}
