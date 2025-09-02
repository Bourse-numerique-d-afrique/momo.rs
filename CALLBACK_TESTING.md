# MTN MOMO Callback Testing

## Overview

MTN MOMO API sends callbacks to **standard HTTP/HTTPS ports (80 and 443)** regardless of environment (sandbox or production). This means integration tests that use callbacks must listen on these privileged ports.

## Requirements

- **Port 80**: Required for HTTP callbacks from MTN
- **Port 443**: Required for HTTPS callbacks from MTN  
- **Root/sudo access**: Required to bind to privileged ports < 1024

## Running Integration Tests

### Method 1: Using the provided script (Recommended)

```bash
# Make script executable (first time only)
chmod +x run_integration_tests.sh

# Run with sudo to bind to ports 80/443
sudo -E ./run_integration_tests.sh
```

### Method 2: Manual execution

```bash
# Set callback URL (optional - defaults to MTN_CALLBACK_HOST)
export CALLBACK_SERVER_URL="https://momotest.boursenumeriquedafrique.com"

# Run with sudo to access privileged ports
sudo -E cargo test --test 2_disbursement -- --nocapture
```

### Method 3: Individual tests

```bash
# Test specific callback functionality
sudo -E cargo test test_deposit_v2 --test 2_disbursement -- --nocapture
sudo -E cargo test test_1_make_payment_successful --test 0_make_payment -- --nocapture
```

## How Callback Testing Works

1. **Test starts**: `CallbackTestHelper::new()` is called
2. **Listeners bind**: Attempts to bind to ports 80 and 443
3. **API call made**: Test makes request to MTN with callback URL
4. **MTN responds**: API returns success/failure immediately
5. **MTN sends callback**: MTN API sends callback to your URL on port 80 or 443
6. **Test waits**: Test waits up to 30 seconds for callback
7. **Verification**: Test verifies callback was received and parsed correctly

## Callback Flow

```
Test → MTN API → Immediate Response
                     ↓
              (Async callback)
                     ↓
MTN API → Port 80/443 → CallbackTestHelper → Test Verification
```

## Environment Variables

- `CALLBACK_SERVER_URL`: Base URL MTN should send callbacks to
  - Default: Uses `MTN_CALLBACK_HOST` or `"https://momotest.boursenumeriquedafrique.com"`
  - Example: `"https://yourdomain.com"`

- `MTN_CALLBACK_HOST`: Host configured during MTN provisioning
  - Used as fallback for `CALLBACK_SERVER_URL`

## Troubleshooting

### "Permission denied" errors
```
❌ Could not start HTTP listener on port 80: Permission denied
```
**Solution**: Run with `sudo -E` to preserve environment variables

### Port already in use
```
❌ Could not start HTTP listener on port 80: Address already in use
```
**Solution**: Stop other services using port 80/443 or kill existing test processes

### Callback timeouts
```
⚠️  Callback timeout or error: Timeout waiting for callback...
```
**Possible causes**:
- MTN API is slow/overloaded
- Firewall blocking callbacks
- DNS/routing issues
- MTN callback URL misconfigured

**Note**: Tests won't fail on callback timeouts - they just warn since the API call itself succeeded.

## Security Notes

- Only run callback tests in development/test environments
- The callback server accepts all connections - don't expose to production traffic
- Consider using Docker or VMs to isolate callback testing
- Callback listeners automatically shut down when tests complete

## Example Test Output

```
🧪 Starting integration tests with MTN callback server support...
📡 MTN will send callbacks to ports 80 and 443 (standard HTTP/HTTPS)
🌐 Callback URL: https://momotest.boursenumeriquedafrique.com

Starting callback listeners on standard ports (MTN requirement)...
✅ Started HTTP callback listener on port 80
✅ Started HTTPS callback listener on port 443
🚀 Callback helper initialized with base URL: https://momotest.boursenumeriquedafrique.com

Using callback URL: https://momotest.boursenumeriquedafrique.com
Waiting for callback for external_id: 328d3ece-a55a-41a5-aa8f-9d1f538d7018
📨 Received HTTPS callback for external_id: 328d3ece-a55a-41a5-aa8f-9d1f538d7018
Received callback: DisbursementDepositV2Success { ... }
```