mod common;

#[cfg(test)]
mod tests {
    use mtnmomo::{Currency, Momo, Party, PartyIdType, RequestToPay};
    use std::env;
    use tokio::sync::OnceCell;

    static MOMO: OnceCell<Momo> = OnceCell::const_new();

    async fn get_momo() -> &'static Momo {
        MOMO.get_or_init(|| async {
            let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");
            let subscription_key =
                env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
            let momo_result =
                Momo::new_with_provisioning(mtn_url, subscription_key, "webhook.site").await;
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

        let request = RequestToPay::new(
            "100".to_string(),
            Currency::EUR,
            payer,
            "test_payer_message".to_string(),
            "test_payee_note".to_string(),
        );

        let request_to_pay_result = collection
            .request_to_pay(
                request,
                Some("http://webhook.site/0e1ea918-075d-4916-8bf2-a8b696cf82f4"),
            )
            .await;

        assert!(request_to_pay_result.is_ok());
    }

    #[tokio::test]
    async fn test_2_make_payment_rejected() {
        let momo = get_momo().await;
        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key =
            env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let collection = momo.collection(primary_key, secondary_key);

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

        let request_to_pay_result = collection
            .request_to_pay(
                request,
                Some("http://webhook.site/0e1ea918-075d-4916-8bf2-a8b696cf82f4"),
            )
            .await;

        // Request should succeed initially (returns 202) even for test numbers that will eventually fail
        assert!(request_to_pay_result.is_ok());
    }

    #[tokio::test]
    async fn test_3_make_payment_expired() {
        let momo = get_momo().await;
        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key =
            env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let collection = momo.collection(primary_key, secondary_key);

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

        let request_to_pay_result = collection
            .request_to_pay(
                request,
                Some("http://webhook.site/0e1ea918-075d-4916-8bf2-a8b696cf82f4"),
            )
            .await;

        // Request should succeed initially (returns 202) even for test numbers that will eventually fail
        assert!(request_to_pay_result.is_ok());
    }

    #[tokio::test]
    async fn test_4_make_payment_ongoing() {
        let momo = get_momo().await;
        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key =
            env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let collection = momo.collection(primary_key, secondary_key);

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

        let request_to_pay_result = collection
            .request_to_pay(
                request,
                Some("http://webhook.site/0e1ea918-075d-4916-8bf2-a8b696cf82f4"),
            )
            .await;

        // Request should succeed initially (returns 202) even for test numbers that will eventually fail
        assert!(request_to_pay_result.is_ok());
    }

    #[tokio::test]
    async fn test_5_make_payment_delayed() {
        let momo = get_momo().await;
        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key =
            env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let collection = momo.collection(primary_key, secondary_key);

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

        let request_to_pay_result = collection
            .request_to_pay(
                request,
                Some("http://webhook.site/0e1ea918-075d-4916-8bf2-a8b696cf82f4"),
            )
            .await;

        // Request should succeed initially (returns 202) even for test numbers that will eventually fail
        assert!(request_to_pay_result.is_ok());
    }
}
