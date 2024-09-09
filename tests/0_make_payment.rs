mod common;

#[cfg(test)]
mod tests {
    use mtnmomo::{Currency, Momo, Party, PartyIdType, RequestToPay};
    use once_cell::sync::Lazy;
    use std::{env, sync::Arc};
    use test_case::test_case;
    use tokio::sync::Mutex;

    static MOMO: Lazy<Arc<Mutex<Option<Momo>>>> = Lazy::new(|| Arc::new(Mutex::new(None)));

    #[tokio::test]
    async fn test_0_make_provisioning() {
        let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");
        let subscription_key =
            env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let momo_result =
            Momo::new_with_provisioning(mtn_url, subscription_key, "webhook.site").await;
        assert!(momo_result.is_ok());
        let momo = momo_result.unwrap();
        let mut _momo = MOMO.lock().await;
        *_momo = Some(momo);
    }

    #[tokio::test]
    #[test_case("+2346733123450", "SUCCESSFULL"; "SuccessfullPayment")]
    #[test_case("46733123450", "INTERNAL_PROCESSING_ERROR"; "RequestToPayPayerFailed")]
    #[test_case("46733123451", "APPROVAL_REJECTED"; "RequestToPayPayerRejected")]
    #[test_case("46733123452", "EXPIRED"; "RequestToPayPayerExpired")]
    #[test_case("46733123453", "ONGOING"; "RequestToPayPayerOngoing")]
    #[test_case("46733123454", "PAYER_DELAYED"; "RequestToPayPayerDelayed")]
    #[test_case("46733123455", "PAYER_NOT_FOUND"; "RequestToPayPayerNotFound")]
    #[test_case("46733123456", "PAYEE_NOT_ALLOWED_TO_RECEIVE"; "RequestToPayPayerPayeeNotAllowedToReceive")]
    #[test_case("46733123457", "NOT_ALLOWED"; "RequestToPayPayerNotAllowed")]
    #[test_case("46733123458", "NOT_ALLOWED_TARGET_ENVIRONMENT"; "RequestToPayPayerNotAllowedTargetEnvironment")]
    #[test_case("46733123459", "INVALID_CALLBACK_URL_HOST"; "RequestToPayPayerInvalidCallbackUrlHost")]
    #[test_case("46733123460", "INVALID_CURRENCY"; "RequestToPayPayerInvalidCurrency")]
    #[test_case("46733123461", "INTERNAL_PROCESSING_ERROR"; "RequestToPayPayerInternalProcessingError")]
    #[test_case("46733123462", "SERVICE_UNAVAILABLE"; "RequestToPayPayerServiceUnavailable")]
    #[test_case("46733123463", "COULD_NOT_PERFORM_TRANSACTION"; "RequestToPayPayerCouldNotPerformTransaction")]
    async fn test_1_make_payment(number: &str, _reason: &str) {
        let momo_lock = MOMO.lock().await;
        let momo = momo_lock.as_ref().unwrap();
        let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
        let secondary_key =
            env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
        let collection = momo.collection(primary_key, secondary_key);

        let payer: Party = Party {
            party_id_type: PartyIdType::MSISDN,
            party_id: number.to_string(),
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
}
