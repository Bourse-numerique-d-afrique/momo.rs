mod common;

#[cfg(test)]
mod tests {

    use mtnmomo::{enums::payer_identification_type::PayerIdentificationType, CashTransferRequest, Currency, Momo, Party, PartyIdType, TransferRequest};
    use std::env;
    use tokio::sync::OnceCell;

    static MOMO: OnceCell<Momo> = OnceCell::const_new();

    async fn get_momo() -> &'static Momo {
        MOMO.get_or_init(|| async {
            let mtn_url = env::var("MTN_URL").expect("MTN_URL must be set");
            let subscription_key =
                env::var("MTN_REMITTANCE_PRIMARY_KEY").expect("PRIMARY_KEY must be set");

            let callback_host = env::var("MTN_CALLBACK_HOST").unwrap_or_else(|_| "webhook.site".to_string());

            println!("Using MTN_CALLBACK_HOST: {}", callback_host);
            let momo_result =
                Momo::new_with_provisioning(mtn_url, subscription_key, &callback_host).await;
            momo_result.expect("Failed to initialize Momo")
        })
        .await
    }

    #[tokio::test]
    async fn test_cash_transfer() {
        let momo = get_momo().await;
        let primary_key = env::var("MTN_REMITTANCE_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key =
            env::var("MTN_REMITTANCE_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let remittance = momo.remittance(primary_key, secondary_key);

        let payee = Party {
            party_id_type: PartyIdType::MSISDN,
            party_id: "256774290781".to_string(),
        };
        let transfer = CashTransferRequest::new(
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

        let call_back_server_url = env::var("CALLBACK_SERVER_URL").unwrap_or_else(|_| "http://webhook.site/0e1ea918-075d-4916-8bf2-a8b696cf82f4".to_string());

        let res = remittance.cash_transfer(transfer, Some(&call_back_server_url)).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_transfer() {
        let momo = get_momo().await;
        let primary_key = env::var("MTN_REMITTANCE_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key =
            env::var("MTN_REMITTANCE_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let remittance = momo.remittance(primary_key, secondary_key);

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

        let transer_result = remittance.transfer(transfer.clone(), Some(&call_back_server_url)).await;
        assert!(transer_result.is_ok());
        assert_eq!(transer_result.unwrap().as_string(), transfer.external_id);
    }
}
