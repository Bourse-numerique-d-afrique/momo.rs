# MTN MoMo Callback Server

A standalone HTTP callback server for handling MTN MoMo payment callbacks. This server listens on port 8500 (localhost only) and processes all types of MTN MoMo callbacks including payments, invoices, disbursements, and remittances. TLS is handled by a reverse proxy (e.g., Caddy2) in production.

## Features

- **📡 Complete Callback Coverage**: Handles all MTN MoMo callback types:
  - **Collection**: Request to pay, invoices, withdrawals, pre-approvals
  - **Disbursements**: Deposits, refunds, transfers
  - **Remittances**: Cash transfers, transfers
- **-pills Health Monitoring**: Built-in `/health` endpoint for uptime checks
- **🛡️ Production Ready**: Graceful shutdown, structured logging, comprehensive error handling
- **🔧 Custom Business Logic**: Easy-to-extend callback handlers for your specific needs

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

### 3. Test the Server

```bash
# Health check
curl http://127.0.0.1:8500/health
# Should return: OK
```

## Configuration

The server runs on localhost:8500 by default. For production deployments, use a reverse proxy like Caddy2 to handle TLS termination.

## API Endpoints

### Health Check
- `GET /health` - Returns "OK" when server is running

### Callback Endpoints

All callback endpoints accept POST requests:

#### Collection Callbacks
- `POST /collection_request_to_pay`
- `POST /collection_request_to_withdraw_v1`
- `POST /collection_request_to_withdraw_v2`
- `POST /collection_invoice`
- `POST /collection_payment`
- `POST /collection_preapproval`

#### Disbursement Callbacks
- `POST /disbursement_deposit_v1`
- `POST /disbursement_deposit_v2`
- `POST /disbursement_refund_v1`
- `POST /disbursement_refund_v2`
- `POST /disbursement_transfer`

#### Remittance Callbacks
- `POST /remittance_cash_transfer`
- `POST /remittance_transfer`

## Usage with MTN MoMo API

When making requests to the MTN MoMo API, set the callback URL to point to your server through the reverse proxy:

```rust
use mtnmomo::{Momo, RequestToPay, Party, PartyIdType, Currency};

// Example: Request to pay with callback
let callback_url = "https://your-domain.com/collection_request_to_pay";
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
        CallbackResponse::InvoiceFailed { external_id, error_reason, .. } => {
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
2024-01-15T10:30:15.127Z INFO [momo_callback_server] Host: 127.0.0.1, Port: 8500
2024-01-15T10:30:15.128Z INFO [momo_callback_server] Binding server to address: 127.0.0.1:8500
2024-01-15T10:30:15.129Z INFO [momo_callback_server] MTN MoMo Callback Server started successfully
2024-01-15T10:30:15.130Z INFO [momo_callback_server] Server is running. Press Ctrl+C to stop.
```

When callbacks are received:
```
2024-01-15T10:31:20.456Z INFO [momo_callback_server] Received callback from 192.168.1.100
2024-01-15T10:31:20.457Z INFO [momo_callback_server] Successfully processed callback
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