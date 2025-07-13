# MTN MoMo Callback Server

A standalone TLS-enabled callback server for handling MTN MoMo payment callbacks. This server listens on port 443 with TLS encryption and processes all types of MTN MoMo callbacks including payments, invoices, disbursements, and remittances.

## Features

- **🔒 TLS/HTTPS Support**: Secure server listening on port 443 with certificate-based encryption
- **📡 Complete Callback Coverage**: Handles all MTN MoMo callback types:
  - **Collection**: Request to pay, invoices, withdrawals, pre-approvals
  - **Disbursements**: Deposits, refunds, transfers
  - **Remittances**: Cash transfers, transfers
- **💊 Health Monitoring**: Built-in `/health` endpoint for uptime checks
- **🛡️ Production Ready**: Graceful shutdown, structured logging, comprehensive error handling
- **⚙️ Environment Configuration**: Configurable certificate paths via environment variables
- **🔧 Custom Business Logic**: Easy-to-extend callback handlers for your specific needs

## Quick Start

### Prerequisites

- Rust 1.70+ installed
- TLS certificate and private key files in PEM format
- MTN MoMo API credentials

### 1. Build the Server

```bash
git clone <your-momo-repo>
cd momo-callback-server
cargo build --release
```

### 2. Prepare TLS Certificates

Place your certificate and key files in the current directory:
- `cert.pem` - Your TLS certificate
- `key.pem` - Your private key

Or set custom paths via environment variables:
```bash
export TLS_CERT_PATH=/path/to/your/cert.pem
export TLS_KEY_PATH=/path/to/your/key.pem
```

### 3. Run the Server

```bash
# Using default certificate paths
./target/release/momo-callback-server

# Or with custom paths
TLS_CERT_PATH=/path/to/cert.pem TLS_KEY_PATH=/path/to/key.pem ./target/release/momo-callback-server
```

### 4. Test the Server

```bash
# Health check
curl https://localhost/health
# Should return: OK
```

## Configuration

The server uses environment variables for configuration:

| Variable | Default | Description |
|----------|---------|-------------|
| `TLS_CERT_PATH` | `cert.pem` | Path to TLS certificate file (PEM format) |
| `TLS_KEY_PATH` | `key.pem` | Path to TLS private key file (PEM format) |

## API Endpoints

### Health Check
- `GET /health` - Returns "OK" when server is running

### Callback Endpoints

All callback endpoints accept POST requests:

#### Collection Callbacks
- `POST /collection_request_to_pay/{callback_type}`
- `POST /collection_request_to_withdraw_v1/{callback_type}`
- `POST /collection_request_to_withdraw_v2/{callback_type}`
- `POST /collection_invoice/{callback_type}`
- `POST /collection_payment/{callback_type}`
- `POST /collection_preapproval/{callback_type}`

#### Disbursement Callbacks
- `POST /disbursement_deposit_v1/{callback_type}`
- `POST /disbursement_deposit_v2/{callback_type}`
- `POST /disbursement_refund_v1/{callback_type}`
- `POST /disbursement_refund_v2/{callback_type}`
- `POST /disbursement_transfer/{callback_type}`

#### Remittance Callbacks
- `POST /remittance_cash_transfer/{callback_type}`
- `POST /remittance_transfer/{callback_type}`

### Callback Types

The `{callback_type}` parameter can be:
- `REQUEST_TO_PAY`
- `REQUEST_TO_WITHDRAW_V1`
- `REQUEST_TO_WITHDRAW_V2`
- `INVOICE`
- `COLLECTION_PAYMENT`
- `COLLECTION_PRE_APPROVAL`
- `DISBURSEMENT_DEPOSIT_V1`
- `DISBURSEMENT_DEPOSIT_V2`
- `DISBURSEMENT_REFUND_V1`
- `DISBURSEMENT_REFUND_V2`
- `DISBURSEMENT_TRANSFER`
- `REMITTANCE_CASH_TRANSFER`
- `REMITTANCE_TRANSFER`

## Usage with MTN MoMo API

When making requests to the MTN MoMo API, set the callback URL to point to your server:

```rust
use mtnmomo::{Momo, RequestToPay, Party, PartyIdType, Currency};

// Example: Request to pay with callback
let callback_url = "https://your-domain.com/collection_request_to_pay/REQUEST_TO_PAY";
let result = collection.request_to_pay(request, Some(callback_url)).await;
```

## Custom Business Logic

The server includes dedicated handlers for different callback types. You can modify these in `src/main.rs`:

### Payment Callbacks
```rust
async fn handle_payment_callback(update: &MomoUpdates) {
    match &update.response {
        CallbackResponse::RequestToPaySuccess { external_id, amount, currency, .. } => {
            // Handle successful payment
            println!("Payment successful: {} {}", amount, currency);
            // Add your business logic here:
            // - Update database
            // - Send notifications
            // - Process orders
        }
        CallbackResponse::RequestToPayFailed { external_id, reason, .. } => {
            // Handle failed payment
            println!("Payment failed: {:?}", reason);
            // Add your failure handling:
            // - Handle refunds
            // - Notify user
            // - Log for investigation
        }
        _ => {}
    }
}
```

### Invoice Callbacks
```rust
async fn handle_invoice_callback(update: &MomoUpdates) {
    match &update.response {
        CallbackResponse::InvoiceSucceeded { external_id, reference_id, .. } => {
            // Handle successful invoice payment
            // Update invoice status, notify customer, etc.
        }
        CallbackResponse::InvoiceFailed { external_id, erron_reason, .. } => {
            // Handle failed invoice
            // Send reminder, update status, etc.
        }
        _ => {}
    }
}
```

## Logging

The server provides comprehensive logging:

```
2024-01-15T10:30:15.123Z INFO [momo_callback_server] MTN MoMo Callback Server starting...
2024-01-15T10:30:15.124Z INFO [momo_callback_server] Loading TLS certificate from: cert.pem
2024-01-15T10:30:15.125Z INFO [momo_callback_server] Loading TLS private key from: key.pem
2024-01-15T10:30:15.126Z INFO [momo_callback_server] TLS configuration loaded successfully
2024-01-15T10:30:15.127Z INFO [momo_callback_server] Host: 0.0.0.0, Port: 443
2024-01-15T10:30:15.128Z INFO [momo_callback_server] Binding to address: 0.0.0.0:443
2024-01-15T10:30:15.129Z INFO [momo_callback_server] MTN MoMo Callback Server started successfully
2024-01-15T10:30:15.130Z INFO [momo_callback_server] Server is running. Press Ctrl+C to stop.
```

When callbacks are received:
```
2024-01-15T10:31:20.456Z INFO [momo_callback_server] Received callback from 192.168.1.100: REQUEST_TO_PAY
2024-01-15T10:31:20.457Z INFO [momo_callback_server] Successfully processed REQUEST_TO_PAY callback
2024-01-15T10:31:20.458Z INFO [momo_callback_server] Processing callback: RequestToPay
2024-01-15T10:31:20.459Z INFO [momo_callback_server] Payment successful - External ID: abc123, Amount: 100 UGX
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
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/momo-callback-server .
COPY cert.pem key.pem ./
EXPOSE 443
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
Environment=TLS_CERT_PATH=/opt/momo-callback-server/cert.pem
Environment=TLS_KEY_PATH=/opt/momo-callback-server/key.pem
ExecStart=/opt/momo-callback-server/momo-callback-server
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

### Reverse Proxy with nginx

If you can't bind to port 443 directly:

```nginx
server {
    listen 443 ssl;
    server_name your-domain.com;
    
    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;
    
    location / {
        proxy_pass http://127.0.0.1:8443;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

Then run the server on port 8443:
```rust
// Modify the port in CallbackServerConfig::default()
port: 8443,
```

## Security

- **TLS 1.2+**: Uses modern TLS encryption for all connections
- **Certificate Validation**: Validates TLS certificates on startup
- **Request Logging**: All callback requests are logged for audit
- **JSON Validation**: Validates callback payloads from MTN MoMo
- **Error Handling**: Failed callbacks are logged with detailed error information

## Troubleshooting

### Common Issues

1. **Certificate not found**
   ```
   Certificate file not found: cert.pem
   ```
   **Solution**: Ensure your certificate file exists and is readable.

2. **Permission denied on port 443**
   ```
   Permission denied (os error 13)
   ```
   **Solution**: Run as root or use `setcap CAP_NET_BIND_SERVICE=+eip ./target/release/momo-callback-server`

3. **TLS handshake failures**
   ```
   TLS handshake error
   ```
   **Solution**: Verify your certificate and key files are valid and match.

### Debug Mode

Run with debug logging:
```bash
RUST_LOG=debug ./target/release/momo-callback-server
```

### Testing Callbacks

Use `curl` to test callback endpoints:

```bash
curl -X POST https://localhost/collection_request_to_pay/REQUEST_TO_PAY \
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
2. Add handling in the appropriate callback handler function
3. Test with the new callback type

### Custom Middleware

Add custom middleware to the server:

```rust
// In create_callback_routes()
.with(your_custom_middleware())
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Support

For issues and support:
1. Check the troubleshooting section
2. Review the logs for error details
3. Open an issue in the repository
4. Contact the maintainers