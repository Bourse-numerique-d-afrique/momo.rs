mod common;
use mtnmomo::{Currency, Momo, Party, PartyIdType, RequestToPay};
use once_cell::sync::Lazy;
use std::{env, sync::Arc};
use tokio::sync::Mutex;

static MOMO: Lazy<Arc<Mutex<Option<Momo>>>> = Lazy::new(|| Arc::new(Mutex::new(None)));

#[tokio::test]
async fn test_0_make_provisioning() {
    let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set");
    let subscription_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
    let momo_result = Momo::new_with_provisioning(
        mtn_url,
        subscription_key,
        "momo.boursenumeriquedafrique.com",
    )
    .await;
    assert!(momo_result.is_ok());
    let momo = momo_result.unwrap();
    let mut _momo = MOMO.lock().await;
    *_momo = Some(momo);
}

#[tokio::test]
async fn test_1_make_payment() {
    let momo_lock = MOMO.lock().await;
    let momo = momo_lock.as_ref().unwrap();
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

    let request_to_pay_result = collection
        .request_to_pay(request, Some("http://momo.boursenumeriquedafrique.com/mtn"))
        .await;

    assert!(request_to_pay_result.is_ok());
}
