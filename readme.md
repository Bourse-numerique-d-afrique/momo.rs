### MOMO.rs is a Rust library for the MOMO payment gateway.
[![build tests](https://github.com/Bourse-numerique-d-afrique/momo.rs/actions/workflows/deployment.yml/badge.svg)](https://github.com/Bourse-numerique-d-afrique/momo.rs/actions/workflows/deployment.yml)
[![crates.io](https://github.com/Bourse-numerique-d-afrique/momo.rs/actions/workflows/publish.yml/badge.svg)](https://github.com/Bourse-numerique-d-afrique/momo.rs/actions/workflows/publish.yml)
[![Crates.io](https://img.shields.io/crates/v/mtnmomo.svg)](https://crates.io/crates/mtnmomo)
[![MIT licensed](https://img.shields.io/badge/License-MIT-yellow.svg)](https://choosealicense.com/licenses/mit/)
[![Docs](https://img.shields.io/badge/docs-yes-brightgreen.svg)](https://docs.rs/mtnmomo/0.1.7/mtnmomo/)

<div align="center">

![MOMO logo](https://raw.githubusercontent.com/Bourse-numerique-d-afrique/momo.rs/master/images/BrandGuid-mtnmomo.svg)

</div>


### Installation
```toml
[dependencies]
mtnmomo = "0.1.7"
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

This library includes an integrated callback server for handling MTN MoMo webhooks. The callback server provides a secure HTTPS endpoint that processes payment notifications from MTN MoMo API.

#### Installation with Callback Server

```toml
[dependencies]
mtnmomo = { version = "0.1.7", features = ["callback-server"] }
```

#### Basic Callback Server Usage

```rust
use mtnmomo::{CallbackServerConfig, start_callback_server};
use futures_util::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure the callback server
    let config = CallbackServerConfig {
        cert_path: "cert.pem".to_string(),
        key_path: "key.pem".to_string(),
        port: 443,
        host: "0.0.0.0".to_string(),
    };

    // Start the callback server
    let mut callback_stream = start_callback_server(config).await?;

    println!("Callback server started on https://0.0.0.0:443");
    println!("Health check available at: https://your-domain.com/health");

    // Process incoming callbacks
    while let Some(callback) = callback_stream.next().await {
        println!("Received callback from: {}", callback.remote_address);
        println!("Callback type: {:?}", callback.update_type);
        println!("Response: {:?}", callback.response);

        // Add your business logic here to handle different callback types
        match callback.update_type {
            mtnmomo::CallbackType::RequestToPay => {
                println!("Processing payment callback");
                // Handle payment completion/failure
            }
            mtnmomo::CallbackType::Invoice => {
                println!("Processing invoice callback");
                // Handle invoice events
            }
            mtnmomo::CallbackType::DisbursementDepositV1 | 
            mtnmomo::CallbackType::DisbursementDepositV2 => {
                println!("Processing disbursement callback");
                // Handle disbursement completion
            }
            _ => {
                println!("Processing other callback type");
            }
        }
    }

    Ok(())
}
```

#### Environment Configuration

The callback server can be configured using environment variables:

```bash
export TLS_CERT_PATH="/path/to/your/cert.pem"
export TLS_KEY_PATH="/path/to/your/key.pem"
```

#### TLS Certificate Setup

For production use, you'll need valid TLS certificates. You can obtain free certificates from Let's Encrypt:

```bash
# Install certbot
sudo apt-get install certbot

# Get certificate for your domain
sudo certbot certonly --standalone -d your-domain.com

# Copy certificates to your application directory
sudo cp /etc/letsencrypt/live/your-domain.com/fullchain.pem cert.pem
sudo cp /etc/letsencrypt/live/your-domain.com/privkey.pem key.pem
```

For development/testing, you can create self-signed certificates:

```bash
# Generate private key
openssl genrsa -out key.pem 2048

# Generate self-signed certificate
openssl req -new -x509 -key key.pem -out cert.pem -days 365
```

#### Callback URLs

When making API calls, use your callback server URLs:

```rust
// For payments
let callback_url = "https://your-domain.com/collection_request_to_pay/REQUEST_TO_PAY";
let result = collection.request_to_pay(request, Some(&callback_url)).await;

// For disbursements  
let callback_url = "https://your-domain.com/disbursement_deposit_v1/DISBURSEMENT_DEPOSIT_V1";
let result = disbursement.deposit(request, Some(&callback_url)).await;
```

#### Available Endpoints

The callback server automatically creates endpoints for all MTN MoMo services:

- **Collection**: `/collection_request_to_pay/{callback_type}`
- **Collection Withdrawals**: `/collection_request_to_withdraw_v1/{callback_type}`, `/collection_request_to_withdraw_v2/{callback_type}`
- **Invoices**: `/collection_invoice/{callback_type}`, `/collection_payment/{callback_type}`
- **Disbursements**: `/disbursement_deposit_v1/{callback_type}`, `/disbursement_deposit_v2/{callback_type}`
- **Remittances**: `/remittance_cash_transfer/{callback_type}`, `/remittance_transfer/{callback_type}`
- **Health Check**: `/health` (GET)

#### Features

- **🔒 TLS/HTTPS Support**: Secure server with certificate-based encryption
- **📡 Complete Callback Coverage**: Handles all MTN MoMo callback types
- **💊 Health Monitoring**: Built-in health check endpoint for load balancers
- **🛡️ Production Ready**: Graceful shutdown, structured logging, comprehensive error handling
- **⚙️ Environment Configuration**: Configurable via environment variables
- **🔧 Extensible**: Easy-to-extend callback handlers for custom business logic
