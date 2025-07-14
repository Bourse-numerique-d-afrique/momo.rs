### MOMO.rs is a Rust library for the MOMO payment gateway.
[![build tests](https://github.com/Bourse-numerique-d-afrique/momo.rs/actions/workflows/deployment.yml/badge.svg)](https://github.com/Bourse-numerique-d-afrique/momo.rs/actions/workflows/deployment.yml)
[![crates.io](https://github.com/Bourse-numerique-d-afrique/momo.rs/actions/workflows/publish.yml/badge.svg)](https://github.com/Bourse-numerique-d-afrique/momo.rs/actions/workflows/publish.yml)
[![Crates.io](https://img.shields.io/crates/v/mtnmomo.svg)](https://crates.io/crates/mtnmomo)
[![MIT licensed](https://img.shields.io/badge/License-MIT-yellow.svg)](https://choosealicense.com/licenses/mit/)
[![Docs](https://img.shields.io/badge/docs-yes-brightgreen.svg)](https://docs.rs/mtnmomo/0.1.4/mtnmomo/)

<div align="center">

![MOMO logo](https://raw.githubusercontent.com/Bourse-numerique-d-afrique/momo.rs/master/images/BrandGuid-mtnmomo.svg)

</div>


### Installation
```toml
[dependencies]
mtnmomo = "0.1.4"
```

or you can use cargo add

```cli
cargo add mtnmomo
```


### MTN Mobile Money API

This package provides for an easy way to connect to MTN MoMo API, it provides for the following products:
- Collection
- Disbursements
- Remittance
- Provisioning in case of sandbox environment

### how to use:
 ``` rust
use mtnmomo::Momo;
use uuid::Uuid;
use dotenv::dotenv;
use std::env;

#[tokio::main]
async fn main() {
dotenv().ok();
let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set"); // https://sandbox.momodeveloper.mtn.com
let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
let secondary_key = env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
let momo = Momo::new_with_provisioning(mtn_url, primary_key.clone(), "webhook.site").await.unwrap();
let collection = momo.collection(primary_key, secondary_key);
}

```
After initializing the Momo struct, you can then use the collection, disbursement or remittance methods to initialize the respective products.
The products have methods that you can use to interact with the API.
For example, to request a payment from a customer, you can use the request_to_pay method of the Collection product.


### important notes:
`mtnmomo::Momo::new_with_provisioning` is used to initialize the Momo struct with the sandbox environment.
`mtnmomo::Momo::new` is used to initialize the Momo struct with the production environment.

### example making a request to pay:
``` rust
use mtnmomo::{Momo, Party, PartyIdType, Currency, RequestToPay};
use uuid::Uuid;
use dotenv::dotenv;
use std::env;

#[tokio::main]
async fn main() {
  dotenv().ok();
  let mtn_url = env::var("MTN_URL").expect("MTN_COLLECTION_URL must be set"); // https://sandbox.momodeveloper.mtn.com
  let primary_key = env::var("MTN_COLLECTION_PRIMARY_KEY").expect("PRIMARY_KEY must be set");
  let secondary_key = env::var("MTN_COLLECTION_SECONDARY_KEY").expect("SECONDARY_KEY must be set");
  let momo = Momo::new_with_provisioning(mtn_url, primary_key.clone(), "webhook.site").await.unwrap();
  let collection = momo.collection(primary_key, secondary_key);

   let payer : Party = Party {
          party_id_type: PartyIdType::MSISDN,
         party_id: "46733123450".to_string(), // Use MTN sandbox test number
     };

  let request = RequestToPay::new("100".to_string(), Currency::EUR, payer, "test_payer_message".to_string(), "test_payee_note".to_string());
  let result = collection.request_to_pay(request, Some("http://webhook.site/callback")).await;
}
```
The above code will request a payment of 100 EUR from the customer with the phone number "46733123450" (MTN sandbox test number).
In the sandbox environment, this test number will simulate a successful payment.
The customer will receive a prompt on their phone to confirm the payment.
If the customer confirms the payment, the payment will be processed and the customer will receive a confirmation message.
If the customer declines the payment, the payment will not be processed and the customer will receive a message informing them that the payment was declined.

### Testing with MTN Sandbox

When using the sandbox environment, you should use MTN's predefined test phone numbers:
- `46733123450` - Successful payment
- `46733123451` - Payment rejection  
- `46733123452` - Payment expiry
- `46733123453` - Ongoing payment
- `46733123454` - Delayed payment (succeeds after 30 seconds)

### Callback Server

This library also includes a callback server for handling MTN MoMo webhooks. See the `momo-callback-server` directory for more details.
