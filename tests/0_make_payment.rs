mod common;

#[cfg(test)]
mod tests {
    use crate::common::CallbackTestHelper;
    use mtnmomo::{enums::{reason::RequestToPayReason, request_to_pay_status::RequestToPayStatus}, Currency, Momo, Party, PartyIdType, RequestToPay};
    use std::env;
    use tokio::sync::OnceCell;
    use futures_util::StreamExt;

    static MOMO: OnceCell<Momo> = OnceCell::const_new();

    async fn get_momo() -> &'static Momo {
        MOMO.get_or_init(|| async {
            let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");
            let subscription_key =
                env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");

            let callback_host = env::var("MTN_CALLBACK_HOST").unwrap_or_else(|_| "webhook.site".to_string());

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

        let mut d = CallbackTestHelper::new().await.expect("Failed to start callback listener");

        let request = RequestToPay::new(
            "100".to_string(),
            Currency::EUR,
            payer,
            "test_payer_message".to_string(),
            "test_payee_note".to_string(),
        );

        let callback_url = env::var("CALLBACK_SERVER_URL").unwrap_or_else(|_| "http://webhook.site/0e1ea918-075d-4916-8bf2-a8b696cf82f4".to_string());



        let request_to_pay_result = collection
            .request_to_pay(
                request.clone(),
                Some(&callback_url),
            )
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
            panic!("Expected RequestToPaySuccess callback, got {:?}", callback.response);
        }
    } else {
        panic!("Did not receive callback");
}


    }

    #[tokio::test]
    async fn test_2_make_payment_rejected() {
        let momo = get_momo().await;
        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key =
            env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let collection = momo.collection(primary_key, secondary_key);

        let mut d = CallbackTestHelper::new().await.expect("Failed to start callback listener");

        let payer: Party = Party {
            party_id_type: PartyIdType::MSISDN,
            party_id: "46733123451".to_string(),
        };

        let request = RequestToPay::new(
            "100".to_string(),
            Currency::EUR,
            payer,
            "test_payer_message".to_string(),
            "test_payee_note".to_string(),
        );

        let call_back_server_url = env::var("CALLBACK_SERVER_URL").unwrap_or_else(|_| "http://webhook.site/0e1ea918-075d-4916-8bf2-a8b696cf82f4".to_string());

        let request_to_pay_result = collection
            .request_to_pay(
                request.clone(),
                Some(&call_back_server_url),
            )
            .await;

        assert!(request_to_pay_result.is_ok());

        if let Some(callback) = d.next().await {
            if let mtnmomo::CallbackResponse::RequestToPayFailed {
                financial_transaction_id,
                external_id,
                amount,
                currency,
                payer,
                payee_note,
                payer_message: _, // ignore if not needed
                status,
                reason
            } = callback.response
            {
                assert_eq!(external_id, request.external_id);
                assert_eq!(amount, request.amount);
                assert_eq!(currency, request.currency.to_string());
                assert_eq!(payer.party_id, request.payer.party_id);
                assert_eq!(payee_note, Some(request.payee_note));
                assert_eq!(status, RequestToPayStatus::FAILED);
                assert_eq!(reason, RequestToPayReason::APPROVALREJECTED);
                assert!(!financial_transaction_id.unwrap().is_empty());

            } else {
                panic!("Expected RequestToPayRejected callback, got {:?}", callback.response);
            }
        }
    }

    #[tokio::test]
    async fn test_3_make_payment_expired() {
        let momo = get_momo().await;
        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key =
            env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let collection = momo.collection(primary_key, secondary_key);

        let mut d = CallbackTestHelper::new().await.expect("Failed to start callback listener");

        let payer: Party = Party {
            party_id_type: PartyIdType::MSISDN,
            party_id: "46733123452".to_string(),
        };

        let request = RequestToPay::new(
            "100".to_string(),
            Currency::EUR,
            payer,
            "test_payer_message".to_string(),
            "test_payee_note".to_string(),
        );

        let call_back_server_url = env::var("CALLBACK_SERVER_URL").unwrap_or_else(|_| "http://webhook.site/0e1ea918-075d-4916-8bf2-a8b696cf82f4".to_string());

        let request_to_pay_result = collection
            .request_to_pay(
                request.clone(),
                Some(&call_back_server_url),
            )
            .await;

        // Request should succeed initially (returns 202) even for test numbers that will eventually fail
        assert!(request_to_pay_result.is_ok());

        if let Some(callback) = d.next().await {
            if let mtnmomo::CallbackResponse::RequestToPayFailed {
                financial_transaction_id,
                external_id,
                amount,
                currency,
                payer,
                payee_note,
                payer_message: _, // ignore if not needed
                status,
                reason
            } = callback.response
            {
                assert_eq!(external_id, request.external_id);
                assert_eq!(amount, request.amount);
                assert_eq!(currency, request.currency.to_string());
                assert_eq!(payer.party_id, request.payer.party_id);
                assert_eq!(payee_note, Some(request.payee_note));
                assert_eq!(status, RequestToPayStatus::FAILED);
                assert_eq!(reason, RequestToPayReason::EXPIRED);
                assert!(!financial_transaction_id.unwrap().is_empty());

            } else {
                panic!("Expected RequestToPayFailed callback, got {:?}", callback.response);
            }
        }
    }

    #[tokio::test]
    async fn test_4_make_payment_ongoing() {
        let momo = get_momo().await;
        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key =
            env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let collection = momo.collection(primary_key, secondary_key);

        let mut d = CallbackTestHelper::new().await.expect("Failed to start callback listener");

        let payer: Party = Party {
            party_id_type: PartyIdType::MSISDN,
            party_id: "46733123453".to_string(),
        };

        let request = RequestToPay::new(
            "100".to_string(),
            Currency::EUR,
            payer,
            "test_payer_message".to_string(),
            "test_payee_note".to_string(),
        );

        let call_back_server_url = env::var("CALLBACK_SERVER_URL").unwrap_or_else(|_| "http://webhook.site/0e1ea918-075d-4916-8bf2-a8b696cf82f4".to_string());

        let request_to_pay_result = collection
            .request_to_pay(
                request.clone(),
                Some(&call_back_server_url),
            )
            .await;

        // Request should succeed initially (returns 202) even for test numbers that will eventually fail
        assert!(request_to_pay_result.is_ok());


        if let Some(callback) = d.next().await {
            if let mtnmomo::CallbackResponse::RequestToPayFailed {
                financial_transaction_id,
                external_id,
                amount,
                currency,
                payer,
                payee_note,
                payer_message: _, // ignore if not needed
                status,
                reason
            } = callback.response
            {
                assert_eq!(external_id, request.external_id);
                assert_eq!(amount, request.amount);
                assert_eq!(currency, request.currency.to_string());
                assert_eq!(payer.party_id, request.payer.party_id);
                assert_eq!(payee_note, Some(request.payee_note));
                assert_eq!(status, RequestToPayStatus::FAILED);
                assert_eq!(reason, RequestToPayReason::ONGOING);
                assert!(!financial_transaction_id.unwrap().is_empty());

            } else {
                panic!("Expected RequestToPayFailed callback, got {:?}", callback.response);
            }
        }
    }

    #[tokio::test]
    async fn test_5_make_payment_delayed() {
        let momo = get_momo().await;
        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key =
            env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let collection = momo.collection(primary_key, secondary_key);

        let mut d = CallbackTestHelper::new().await.expect("Failed to start callback listener");

        let payer: Party = Party {
            party_id_type: PartyIdType::MSISDN,
            party_id: "46733123454".to_string(),
        };

        let request = RequestToPay::new(
            "100".to_string(),
            Currency::EUR,
            payer,
            "test_payer_message".to_string(),
            "test_payee_note".to_string(),
        );

        let call_back_server_url = env::var("CALLBACK_SERVER_URL").unwrap_or_else(|_| "http://webhook.site/0e1ea918-075d-4916-8bf2-a8b696cf82f4".to_string());

        let request_to_pay_result = collection
            .request_to_pay(
                request.clone(),
                Some(&call_back_server_url),
            )
            .await;

        // Request should succeed initially (returns 202) even for test numbers that will eventually fail
        assert!(request_to_pay_result.is_ok());

        if let Some(callback) = d.next().await {
            if let mtnmomo::CallbackResponse::RequestToPayFailed {
                financial_transaction_id,
                external_id,
                amount,
                currency,
                payer,
                payee_note,
                payer_message: _, // ignore if not needed
                status,
                reason
            } = callback.response
            {
                assert_eq!(external_id, request.external_id);
                assert_eq!(amount, request.amount);
                assert_eq!(currency, request.currency.to_string());
                assert_eq!(payer.party_id, request.payer.party_id);
                assert_eq!(payee_note, Some(request.payee_note));
                assert_eq!(status, RequestToPayStatus::FAILED);
                assert_eq!(reason, RequestToPayReason::PAYERDELAYED);
                assert!(!financial_transaction_id.unwrap().is_empty());

            } else {
                panic!("Expected RequestToPayFailed callback, got {:?}", callback.response);
            }
        }
    }

    #[tokio::test]
    async fn test_request_to_withdraw_v1() {
        let momo = get_momo().await;
        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key =
            env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let collection = momo.collection(primary_key, secondary_key);
        let mut d = CallbackTestHelper::new().await.expect("Failed to start callback listener");

        let payer: Party = Party {
            party_id_type: PartyIdType::MSISDN,
            party_id: "467331234534".to_string(),
        };
        let request = RequestToPay::new(
            "100.0".to_string(),
            Currency::EUR,
            payer,
            "test_payer_message".to_string(),
            "test_payee_note".to_string(),
        );

        let call_back_server_url = env::var("CALLBACK_SERVER_URL").unwrap_or_else(|_| "http://webhook.site/0e1ea918-075d-4916-8bf2-a8b696cf82f4".to_string());

        let res = collection
            .request_to_withdraw_v1(request, Some(&call_back_server_url))
            .await
            .expect("Error requesting to withdraw");
        assert_ne!(res.as_str().len(), 0);

    }

    #[tokio::test]
    async fn test_request_to_withdraw_v2() {
        let momo = get_momo().await;
        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key =
            env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let collection = momo.collection(primary_key, secondary_key);

        let payer: Party = Party {
            party_id_type: PartyIdType::MSISDN,
            party_id: "+242064818006".to_string(),
        };
        let request = RequestToPay::new(
            "100".to_string(),
            Currency::EUR,
            payer,
            "test_payer_message".to_string(),
            "test_payee_note".to_string(),
        );

        let call_back_server_url = env::var("CALLBACK_SERVER_URL").unwrap_or_else(|_| "http://webhook.site/0e1ea918-075d-4916-8bf2-a8b696cf82f4".to_string());

        let res = collection
            .request_to_withdraw_v2(request, Some(&call_back_server_url))
            .await
            .expect("Error requesting to withdraw");
        assert_ne!(res.as_str().len(), 0);
    }

    #[tokio::test]
    async fn test_create_payment() {
        let momo = get_momo().await;
        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key =
            env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let collection = momo.collection(primary_key, secondary_key);

        let payment = mtnmomo::CreatePaymentRequest::new(
            mtnmomo::Money {
                amount: "100".to_string(),
                currency: Currency::EUR.to_string(),
            },
            "561551442".to_string(),
            "WaterProvider".to_string(),
            "203".to_string(),
            "Monthly Payments".to_string(),
            "788".to_string(),
            "Thank You ".to_string(),
            "Thank You".to_string(),
            2,
            true,
        );

        let call_back_server_url = env::var("CALLBACK_SERVER_URL").unwrap_or_else(|_| "http://webhook.site/0e1ea918-075d-4916-8bf2-a8b696cf82f4".to_string());

        let res = collection
            .create_payments(payment, Some(&call_back_server_url))
            .await
            .expect("Error creating payment");
        assert_ne!(res.as_str().len(), 0);
    }

    #[tokio::test]
    async fn test_pre_approval() {
        let momo = get_momo().await;
        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key =
            env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let collection = momo.collection(primary_key, secondary_key);

        let user: Party = Party {
            party_id_type: PartyIdType::MSISDN,
            party_id: "+242064818006".to_string(),
        };
        let preapproval = mtnmomo::PreApprovalRequest {
            payer: user,
            payer_currency: Currency::EUR.to_string(),
            payer_message: "".to_string(),
            validity_time: 3600,
        };

        let call_back_server_url = env::var("CALLBACK_SERVER_URL").unwrap_or_else(|_| "http://webhook.site/0e1ea918-075d-4916-8bf2-a8b696cf82f4".to_string());

        let res = collection.pre_approval(preapproval, Some(&call_back_server_url)).await;
        if res.is_ok() {
            assert!(true);
        }
    }
}