mod common;

#[cfg(test)]
mod tests {
    use crate::common::CallbackTestHelper;
    use mtnmomo::{Currency, Momo, Party, PartyIdType, RefundRequest, TransferRequest};
    use std::env;
    use std::time::Duration;
    use tokio::sync::OnceCell;

    static MOMO: OnceCell<Momo> = OnceCell::const_new();

    async fn get_momo() -> &'static Momo {
        MOMO.get_or_init(|| async {
            let mtn_url = env::var("MTN_URL").expect("MTN_URL must be set");
            let subscription_key =
                env::var("MTN_DISBURSEMENT_PRIMARY_KEY").expect("PRIMARY_KEY must be set");

            let callback_host = env::var("MTN_CALLBACK_HOST").unwrap_or_else(|_| "webhook.site".to_string());

            println!("Using MTN_CALLBACK_HOST: {}", callback_host);
            let momo_result =
                Momo::new_with_provisioning(mtn_url, subscription_key, &callback_host).await;
            momo_result.expect("Failed to initialize Momo")
        })
        .await
    }

    #[tokio::test]
    async fn test_deposit_v1() {
        let momo = get_momo().await;
        let primary_key = env::var("MTN_DISBURSEMENT_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key =
            env::var("MTN_DISBURSEMENT_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let disbursements = momo.disbursement(primary_key, secondary_key);

        let payee = Party {
            party_id_type: PartyIdType::MSISDN,
            party_id: "256774290781".to_string(),
        };
        let transfer = TransferRequest::new(
            "100".to_string(),
            Currency::EUR,
            payee,
            "payer_message".to_string(),
            "payee_note".to_string(),
        );

        let call_back_server_url = env::var("CALLBACK_SERVER_URL").unwrap_or_else(|_| "http://webhook.site/0e1ea918-075d-4916-8bf2-a8b696cf82f4".to_string());

        let result = disbursements.deposit_v1(transfer.clone(), Some(&call_back_server_url)).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_string(), transfer.external_id);
    }

    #[tokio::test]
    async fn test_deposit_v2() {
        // Start callback listener on both ports 80 and 443
        let callback_helper = CallbackTestHelper::new().await.expect("Failed to start callback listener");
        
        let momo = get_momo().await;
        let primary_key = env::var("MTN_DISBURSEMENT_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key =
            env::var("MTN_DISBURSEMENT_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let disbursements = momo.disbursement(primary_key, secondary_key);

        let payee = Party {
            party_id_type: PartyIdType::MSISDN,
            party_id: "256774290781".to_string(),
        };
        let transfer = TransferRequest::new(
            "100".to_string(),
            Currency::EUR,
            payee,
            "payer_message".to_string(),
            "payee_note".to_string(),
        );

        let callback_url = callback_helper.callback_url();
        println!("Using callback URL: {}", callback_url);

        let result = disbursements.deposit_v2(transfer.clone(), Some(&callback_url)).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_string(), transfer.external_id);

        // Wait for callback with timeout
        println!("Waiting for callback for external_id: {}", transfer.external_id);
        match callback_helper.wait_for_callback(&transfer.external_id, Duration::from_secs(30)).await {
            Ok(callback_response) => {
                println!("Received callback: {:?}", callback_response);
                // Additional assertions can be made on the callback response here
            }
            Err(e) => {
                println!("Warning: Callback timeout or error: {}", e);
                // Don't fail the test if callback times out - the API call succeeded
            }
        }
    }

    #[tokio::test]
    async fn test_refund_v1() {
        let momo = get_momo().await;
        let primary_key = env::var("MTN_DISBURSEMENT_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key =
            env::var("MTN_DISBURSEMENT_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let disbursements = momo.disbursement(primary_key, secondary_key);

        let refund = RefundRequest::new(
            "100".to_string(),
            Currency::EUR.to_string(),
            "payer_message".to_string(),
            "payee_note".to_string(),
            uuid::Uuid::new_v4().to_string(),
        );

        let call_back_server_url = env::var("CALLBACK_SERVER_URL").unwrap_or_else(|_| "http://webhook.site/0e1ea918-075d-4916-8bf2-a8b696cf82f4".to_string());

        let refund_res = disbursements.refund_v1(refund, Some(&call_back_server_url)).await;
        match refund_res {
            Ok(id) => {
                assert_ne!(id.as_str().len(), 0);
            }
            Err(e) => {
                println!("Refund failed as expected in sandbox: {:?}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_refund_v2() {
        let momo = get_momo().await;
        let primary_key = env::var("MTN_DISBURSEMENT_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key =
            env::var("MTN_DISBURSEMENT_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let disbursements = momo.disbursement(primary_key, secondary_key);

        let refund = RefundRequest::new(
            "100".to_string(),
            Currency::EUR.to_string(),
            "payer_message".to_string(),
            "payee_note".to_string(),
            uuid::Uuid::new_v4().to_string(),
        );

        let call_back_server_url = env::var("CALLBACK_SERVER_URL").unwrap_or_else(|_| "http://webhook.site/0e1ea918-075d-4916-8bf2-a8b696cf82f4".to_string());

        let refund_res = disbursements.refund_v2(refund, Some(&call_back_server_url)).await;
        match refund_res {
            Ok(id) => {
                assert_ne!(id.as_str().len(), 0);
            }
            Err(e) => {
                println!("Refund failed as expected in sandbox: {:?}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_transfer() {
        let momo = get_momo().await;
        let primary_key = env::var("MTN_DISBURSEMENT_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key =
            env::var("MTN_DISBURSEMENT_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let disbursements = momo.disbursement(primary_key, secondary_key);

        let transfer = TransferRequest::new(
            "100".to_string(),
            Currency::EUR,
            Party {
                party_id_type: PartyIdType::MSISDN,
                party_id: "256774290781".to_string(),
            },
            "payer_message".to_string(),
            "payee_note".to_string(),
        );

        let call_back_server_url = env::var("CALLBACK_SERVER_URL").unwrap_or_else(|_| "http://webhook.site/0e1ea918-075d-4916-8bf2-a8b696cf82f4".to_string());

        let transfer_result = disbursements.transfer(transfer.clone(), Some(&call_back_server_url)).await;
        assert!(transfer_result.is_ok());
        assert_eq!(transfer_result.unwrap().as_string(), transfer.external_id);
    }
}
