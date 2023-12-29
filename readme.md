### MOMO.rs is a Rust library for the MOMO payment gateway.
[![Testing the application](https://github.com/Bourse-numerique-d-afrique/momo.rs/actions/workflows/deployment.yml/badge.svg)](https://github.com/Bourse-numerique-d-afrique/momo.rs/actions/workflows/deployment.yml)
<p align="center">
  <img src="https://github.com/Bourse-numerique-d-afrique/momo.rs/blob/master/images/BrandGuid-mtnmomo.svg" alt="MOMO logo">
</p>


### Installation
```toml
[dependencies]
mtnmomo = "0.1.0"
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
```rust
use mtnmomo::Momo;
use mtnmomo::Environment;
use uuid::Uuid;

#[tokio::main]
async fn main() {
   let api_user = Uuid::new_v4().to_string();
   let api_key = Uuid::new_v4().to_string();
   let mtn_url = "https://sandbox.momodeveloper.mtn.com";
   let momo = Momo::new(mtn_url.to_string(), api_user, Environment::Sandbox, None).await.unwrap();
   let collection = momo.collection(api_user, api_key);
}

```
After initializing the Momo struct, you can then use the collection, disbursement or remittance methods to initialize the respective products.
The products have methods that you can use to interact with the API.
For example, to request a payment from a customer, you can use the request_to_pay method of the Collection product.

```rust
use mtnmomo::Momo;
use mtnmomo::Environment;
use uuid::Uuid;
use mtnmomo::structs::party::Party;
use mtnmomo::requests::request_to_pay::RequestToPay;

#[tokio::main]
async fn main() {
  let api_user = Uuid::new_v4().to_string();
  let api_key = Uuid::new_v4().to_string();
  let mtn_url = "https://sandbox.momodeveloper.mtn.com";
  let momo = Momo::new(mtn_url.to_string(), api_user, Environment::Sandbox, None).await.unwrap();
  let collection = momo.collection(api_user, api_key);

   let payer : Party = Party {
          party_id_type: "MSISDN".to_string(),
         party_id: "msisdn".to_string(),
     };

  let request = RequestToPay::new("100".to_string(), Currency::EUR, payer, "test_payer_message".to_string(), "test_payee_note".to_string());
  let result = collection.request_to_pay(request).await;
  assert_eq!(result.is_ok(), true);
}
```
The above code will request a payment of 100 EUR from the customer with the phone number "msisdn".
The customer will receive a prompt on their phone to confirm the payment.
If the customer confirms the payment, the payment will be processed and the customer will receive a confirmation message.
If the customer declines the payment, the payment will not be processed and the customer will receive a message informing them that the payment was declined.
The request_to_pay method returns a Result<RequestToPayResponse, Box<dyn Error>>.
