# MTN MoMo Callback Server

A standalone HTTP callback server for handling MTN MoMo payment callbacks. This server listens on port 8500 (localhost only) and processes all types of MTN MoMo callbacks including payments, invoices, disbursements, and remittances. TLS is handled by a reverse proxy (e.g., Caddy2) in production.

## Features

- **📡 Complete Callback Coverage**: Handles all MTN MoMo callback types:
  - **Collection**: Request to pay, invoices, withdrawals, pre-approvals
  - **Disbursements**: Deposits, refunds, transfers
  - **Remittances**: Cash transfers, transfers
- **🩺 Health Monitoring**: Built-in `/health` endpoint for uptime checks
- **🛡️ Production Ready**: Graceful shutdown, structured logging, comprehensive error handling
- **🔧 Library and Binary**: Can be used as both a standalone binary and a library component
- **🔌 Stream-based Processing**: Process callbacks as a stream for easy integration

## Quick Start

### Prerequisites

- Rust 1.70+ installed
- MTN MoMo API credentials

### 1. Build the Server

```bash
git clone <your-momo-repo>
cd momo-callback-server
cargo build --release
```

### 2. Run the Server

```bash
# Run the server
./target/release/momo-callback-server
```

Or if using as a library in your own application:

```rust
use momo_callback_server::{CallbackServerConfig, start_callback_server};
use futures_util::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = CallbackServerConfig::default();
    let mut callback_stream = start_callback_server(config).await?;

    println!("Server started, processing callbacks...");

    while let Some(callback) = callback_stream.next().await {
        println!("Received callback: {:?}", callback.response);
        // Process the callback according to your business logic
    }

    Ok(())
}
```

### 3. Test the Server

```bash
# Health check
curl http://127.0.0.1:8500/health
# Should return: OK
```

## Configuration

The server runs on localhost:8500 by default. You can customize the configuration:

```rust
use momo_callback_server::CallbackServerConfig;

let config = CallbackServerConfig {
    http_port: 8500,
    host: "127.0.0.1".to_string(),
};
```

For production deployments, use a reverse proxy like Caddy2 to handle TLS termination.

## API Endpoints

### Health Check
- `GET /health` - Returns "OK" when server is running

### Callback Endpoints

All callback endpoints accept POST and PUT requests:

#### Collection Callbacks
- `POST/PUT /collection_request_to_pay`
- `POST/PUT /collection_request_to_withdraw_v1`
- `POST/PUT /collection_request_to_withdraw_v2`
- `POST/PUT /collection_invoice`
- `POST/PUT /collection_payment`
- `POST/PUT /collection_preapproval`

#### Disbursement Callbacks
- `POST/PUT /disbursement_deposit_v1`
- `POST/PUT /disbursement_deposit_v2`
- `POST/PUT /disbursement_refund_v1`
- `POST/PUT /disbursement_refund_v2`
- `POST/PUT /disbursement_transfer`

#### Remittance Callbacks
- `POST/PUT /remittance_cash_transfer`
- `POST/PUT /remittance_transfer`

## Usage with MTN MoMo API

When making requests to the MTN MoMo API, set the callback URL to point to your server through the reverse proxy:

```rust
use mtnmomo::{Momo, RequestToPay, Party, PartyIdType, Currency};

// Example: Request to pay with callback
let callback_url = "https://your-domain.com/collection_request_to_pay";
let result = collection.request_to_pay(request, Some(callback_url)).await;
```

## Custom Business Logic

The server returns a stream of `MomoUpdates` that you can process in your application:

```rust
use momo_callback_server::{CallbackServerConfig, start_callback_server};
use mtnmomo::CallbackResponse;
use futures_util::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = CallbackServerConfig::default();
    let mut callback_stream = start_callback_server(config).await?;

    while let Some(update) = callback_stream.next().await {
        match &update.response {
            CallbackResponse::RequestToPaySuccess { 
                external_id, 
                amount, 
                currency, 
                .. 
            } => {
                // Handle successful payment
                println!("Payment successful: {} {} {}", external_id, amount, currency);
                // Add your business logic here
            }
            CallbackResponse::RequestToPayFailed { 
                external_id, 
                reason, 
                .. 
            } => {
                // Handle failed payment
                println!("Payment failed: {} {:?}", external_id, reason);
                // Add your failure handling
            }
            _ => {
                // Handle other callback types
                println!("Received callback: {:?}", update.response);
            }
        }
    }

    Ok(())
}
```

## Logging

The server provides comprehensive logging:

```
2024-01-15T10:30:15.123Z INFO [momo_callback_server] MTN MoMo Callback Server starting...
2024-01-15T10:30:15.127Z INFO [momo_callback_server] Host: 127.0.0.1, Port: 8500
2024-01-15T10:30:15.128Z INFO [momo_callback_server] Binding server to address: 127.0.0.1:8500
2024-01-15T10:30:15.129Z INFO [momo_callback_server] MTN MoMo Callback Server started successfully
2024-01-15T10:30:15.130Z INFO [momo_callback_server] Server is running. Press Ctrl+C to stop.
```

When callbacks are received:
```
2024-01-15T10:31:20.456Z INFO [momo_callback_server] Received callback from 192.168.1.100
2024-01-15T10:31:20.457Z INFO [momo_callback_server] Raw callback body: {"financialTransactionId":"123456","externalId":"test-payment-001","amount":"100","currency":"UGX","status":"SUCCESSFUL"}
2024-01-15T10:31:20.458Z INFO [momo_callback_server] Successfully processed callback
```

## Production Deployment

### Docker Deployment

Create a `Dockerfile`:

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/momo-callback-server ./
EXPOSE 8500
CMD ["./momo-callback-server"]
```

### systemd Service

Create `/etc/systemd/system/momo-callback-server.service`:

```ini
[Unit]
Description=MTN MoMo Callback Server
After=network.target

[Service]
Type=simple
User=momo
WorkingDirectory=/opt/momo-callback-server
ExecStart=/opt/momo-callback-server/momo-callback-server
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

### Reverse Proxy with Caddy2

Use Caddy2 as a reverse proxy to handle TLS termination:

```caddy
your-domain.com {
    reverse_proxy 127.0.0.1:8500
}
```

## Security

- **Localhost Only**: By default, the server only binds to localhost for security
- **Request Logging**: All callback requests are logged for audit
- **JSON Validation**: Validates callback payloads from MTN MoMo
- **Error Handling**: Failed callbacks are logged with detailed error information
- **CORS Support**: Built-in CORS middleware for web integration

## Troubleshooting

### Common Issues

1. **Permission denied on port 8500**
   ```
   Permission denied (os error 13)
   ```
   **Solution**: Port 8500 is a non-privileged port, so this shouldn't happen. Check your system configuration.

### Debug Mode

Run with debug logging:
```bash
RUST_LOG=debug ./target/release/momo-callback-server
```

### Testing Callbacks

Use `curl` to test callback endpoints:

```bash
curl -X POST http://127.0.0.1:8500/collection_request_to_pay \
  -H "Content-Type: application/json" \
  -d '{
    "financialTransactionId": "123456",
    "externalId": "test-payment-001",
    "amount": "100",
    "currency": "UGX",
    "payer": {
      "partyIdType": "MSISDN",
      "partyId": "+256123456789"
    },
    "payeeNote": "Test payment",
    "payerMessage": "Test message",
    "status": "SUCCESSFUL"
  }'
```

## Development

### Adding New Callback Types

1. Update the `CallbackResponse` enum in the mtnmomo library
2. Add handling in your callback processing code
3. Test with the new callback type

### Custom Middleware

Add custom middleware when using as a library:

```rust
use momo_callback_server::create_callback_routes;
use poem::{Route, Server};

let app = create_callback_routes()
    .with(poem::middleware::Tracing::default())
    .with(poem::middleware::Cors::new())
    .with(poem::middleware::Compression::default())
    .with(poem::middleware::RequestId::default())
    // Add your custom middleware here
    .with(your_custom_middleware());
```

## Library Usage

The callback server can be used as a library component in your own applications:

```rust
use momo_callback_server::{CallbackServerConfig, start_callback_server};
use futures_util::StreamExt;

async fn run_callback_server() -> Result<(), Box<dyn std::error::Error>> {
    let config = CallbackServerConfig::default();
    let mut callback_stream = start_callback_server(config).await?;
    
    tokio::spawn(async move {
        while let Some(callback) = callback_stream.next().await {
            // Process callbacks in a separate task
            tokio::spawn(async move {
                process_callback(callback).await;
            });
        }
    });
    
    Ok(())
}

async fn process_callback(callback: mtnmomo::MomoUpdates) {
    // Your callback processing logic here
    println!("Processing callback from: {}", callback.remote_address);
}
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Support

For issues and support:
1. Check the troubleshooting section
2. Review the logs for error details
3. Open an issue in the repository
4. Contact the maintainers