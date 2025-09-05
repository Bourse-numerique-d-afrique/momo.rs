### MOMO.rs is a Rust library for the MOMO payment gateway.
[![build tests](https://github.com/Bourse-numerique-d-afrique/momo.rs/actions/workflows/deployment.yml/badge.svg)](https://github.com/Bourse-numerique-d-afrique/momo.rs/actions/workflows/deployment.yml)
[![crates.io](https://github.com/Bourse-numerique-d-afrique/momo.rs/actions/workflows/publish.yml/badge.svg)](https://github.com/Bourse-numerique-d-afrique/momo.rs/actions/workflows/publish.yml)
[![Crates.io](https://img.shields.io/crates/v/mtnmomo.svg)](https://crates.io/crates/mtnmomo)
[![MIT licensed](https://img.shields.io/badge/License-MIT-yellow.svg)](https://choosealicense.com/licenses/mit/)
[![Docs](https://img.shields.io/badge/docs-yes-brightgreen.svg)](https://docs.rs/mtnmomo/0.1.9/mtnmomo/)

<div align="center">

![MOMO logo](https://raw.githubusercontent.com/Bourse-numerique-d-afrique/momo.rs/master/images/BrandGuid-mtnmomo.svg)

</div>


### Installation
```toml
[dependencies]
mtnmomo = "0.1.9"
```

or you can use cargo add

```cli
cargo add mtnmomo
```

### Skipping Integration Tests

This library includes integration tests that require access to the MTN MoMo API. To skip these tests during development, you can use the `skip-integration-tests` feature:

```bash
# Run unit tests only (skip integration tests)
cargo test --features skip-integration-tests

# Or use the make command
make unit_test
```

To run the full integration tests (requires MTN MoMo API credentials):
```bash
# Run all tests including integration tests
make integration_test
# or
cargo test --test '*'
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

This library includes an optional callback server feature for handling MTN MoMo webhooks. The callback server runs on port 8500 (localhost only) and processes payment notifications from MTN MoMo API.

#### Installation

To use the callback server, enable the `callback-server` feature:

```toml
[dependencies]
mtnmomo = { version = "0.1.9", features = ["callback-server"] }
```

#### Using the Callback Server

You can integrate the callback server directly into your application:

```rust
use mtnmomo::{CallbackServerConfig, start_callback_server};
use futures_util::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure the callback server (defaults to localhost:8500)
    let config = CallbackServerConfig::default();
    
    // Start the callback server
    let mut callback_stream = start_callback_server(config).await?;

    println!("Callback server started on http://127.0.0.1:8500");
    println!("Health check available at: http://127.0.0.1:8500/health");

    // Process incoming callbacks
    while let Some(update) = callback_stream.next().await {
        println!("Received callback from: {}", update.remote_address);
        println!("Response: {:?}", update.response);

        // Add your business logic here to handle different callback types
        match &update.response {
            mtnmomo::CallbackResponse::RequestToPaySuccess { 
                external_id, 
                amount, 
                currency, 
                .. 
            } => {
                println!("Processing successful payment: {} {} {}", external_id, amount, currency);
                // Handle payment completion
            }
            mtnmomo::CallbackResponse::RequestToPayFailed { 
                external_id, 
                reason, 
                .. 
            } => {
                println!("Processing failed payment: {} {:?}", external_id, reason);
                // Handle payment failure
            }
            _ => {
                println!("Processing other callback type");
            }
        }
    }

    Ok(())
}
```

#### Callback URLs

When making API calls, use your callback server URLs through your reverse proxy:

```rust
// For payments
let callback_url = "https://your-domain.com/collection_request_to_pay";
let result = collection.request_to_pay(request, Some(callback_url)).await;

// For disbursements  
let callback_url = "https://your-domain.com/disbursement_deposit_v1";
let result = disbursement.deposit(request, Some(callback_url)).await;
```

#### Available Endpoints

The callback server provides endpoints for all MTN MoMo services:

- **Collection**: 
  - `POST/PUT /collection_request_to_pay`
  - `POST/PUT /collection_request_to_withdraw_v1`
  - `POST/PUT /collection_request_to_withdraw_v2`
  - `POST/PUT /collection_invoice`
  - `POST/PUT /collection_payment`
  - `POST/PUT /collection_preapproval`

- **Disbursements**:
  - `POST/PUT /disbursement_deposit_v1`
  - `POST/PUT /disbursement_deposit_v2`
  - `POST/PUT /disbursement_refund_v1`
  - `POST/PUT /disbursement_refund_v2`
  - `POST/PUT /disbursement_transfer`

- **Remittances**:
  - `POST/PUT /remittance_cash_transfer`
  - `POST/PUT /remittance_transfer`

- **Health Check**: `GET /health`

#### Caddy2 Reverse Proxy Configuration

For production deployments, use Caddy2 as a reverse proxy to handle TLS termination. Caddy2 automatically provisions and renews TLS certificates from Let's Encrypt.

1. Install Caddy2:
```bash
# For Ubuntu/Debian
sudo apt install -y debian-keyring debian-archive-keyring apt-transport-https
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' | sudo gpg --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/debian.deb.txt' | sudo tee /etc/apt/sources.list.d/caddy-stable.list
sudo apt update
sudo apt install caddy
```

2. Create a Caddyfile:
```caddy
# /etc/caddy/Caddyfile
your-domain.com {
    reverse_proxy 127.0.0.1:8500
}
```

3. Start Caddy:
```bash
sudo systemctl start caddy
sudo systemctl enable caddy
```

With this configuration:
- Caddy2 will automatically obtain a TLS certificate for `your-domain.com`
- All HTTPS traffic to `your-domain.com` will be forwarded to the callback server on port 8500
- The callback server receives plain HTTP traffic but clients connect via HTTPS

#### Testing with Caddy2

For local testing with a custom domain, you can use Caddy2 with a hosts file entry:

1. Add an entry to your hosts file:
```bash
# /etc/hosts
127.0.0.1 test.domain.com
```

2. Create a Caddyfile for local testing:
```caddy
# /etc/caddy/Caddyfile
test.domain.com {
    reverse_proxy 127.0.0.1:8500
    tls internal
}
```

3. Restart Caddy:
```bash
sudo systemctl restart caddy
```

Now you can access your callback server at `https://test.domain.com` with a valid TLS certificate (internal CA for local testing).

#### Features

- **📡 Complete Callback Coverage**: Handles all MTN MoMo callback types
- **🩺 Health Monitoring**: Built-in health check endpoint for load balancers
- **🛡️ Production Ready**: Graceful shutdown, structured logging, comprehensive error handling
- **🔧 Library Integration**: Can be integrated directly into your application
- **🔌 Stream-based Processing**: Process callbacks as a stream for easy integration
- **🔒 TLS with Caddy2**: Production-ready TLS termination with automatic certificate management
