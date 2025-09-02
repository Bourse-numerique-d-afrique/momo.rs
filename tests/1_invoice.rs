mod common;

#[cfg(test)]
mod tests {
    use mtnmomo::{Currency, InvoiceRequest, Momo, Party, PartyIdType};
    use std::env;
    use tokio::sync::OnceCell;

    static MOMO: OnceCell<Momo> = OnceCell::const_new();

    async fn get_momo() -> &'static Momo {
        MOMO.get_or_init(|| async {
            let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");
            let subscription_key =
                env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");

            let callback_host = env::var("MTN_CALLBACK_HOST").unwrap_or_else(|_| "webhook.site".to_string());

            println!("Using MTN_CALLBACK_HOST: {}", callback_host);
            let momo_result =
                Momo::new_with_provisioning(mtn_url, subscription_key, &callback_host).await;
            momo_result.expect("Failed to initialize Momo")
        })
        .await
    }

    #[tokio::test]
    async fn test_create_and_cancel_invoice() {
        let momo = get_momo().await;
        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key =
            env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let collection = momo.collection(primary_key, secondary_key);

        let payer: Party = Party {
            party_id_type: PartyIdType::MSISDN,
            party_id: "22997108557".to_string(),
        };

        let payee: Party = Party {
            party_id_type: PartyIdType::MSISDN,
            party_id: "22997108557".to_string(),
        };

        let invoice = InvoiceRequest::new(
            "100".to_string(),
            Currency::EUR.to_string(),
            "100".to_string(),
            payer,
            payee,
            "test".to_string(),
        );

        let call_back_server_url = env::var("CALLBACK_SERVER_URL").unwrap_or_else(|_| "http://webhook.site/0e1ea918-075d-4916-8bf2-a8b696cf82f4".to_string());

        let invoice_id = collection
            .create_invoice(invoice, Some(&call_back_server_url))
            .await
            .expect("Error creating invoice");

        let res = collection
            .cancel_invoice(&invoice_id.as_str(), Some(&call_back_server_url))
            .await;
        assert!(res.is_ok());
    }
}
