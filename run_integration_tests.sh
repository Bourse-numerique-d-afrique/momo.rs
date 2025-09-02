#!/bin/bash

# Integration test runner with callback server support
# MTN MOMO API always sends callbacks to ports 80 and 443

# Set callback server configuration
export CALLBACK_SERVER_URL="https://momotest.boursenumeriquedafrique.com"

echo "🧪 Starting integration tests with MTN callback server support..."
echo "📡 MTN will send callbacks to ports 80 and 443 (standard HTTP/HTTPS)"
echo "🌐 Callback URL: $CALLBACK_SERVER_URL"
echo ""
echo "⚠️  Note: This requires binding to privileged ports 80 and 443"
echo "💡 Run with sudo to ensure callback listeners can bind to these ports:"
echo ""

# Check if running as root
if [ "$EUID" -eq 0 ]; then
    echo "✅ Running as root - can bind to ports 80 and 443"
    cargo test --test 2_disbursement -- --nocapture
else
    echo "⚠️  Not running as root. Callbacks may fail to bind to ports 80/443"
    echo "🔧 To run with proper permissions:"
    echo "   sudo -E ./run_integration_tests.sh"
    echo ""
    echo "🚀 Attempting to run tests anyway..."
    cargo test --test 2_disbursement -- --nocapture
fi

echo ""
echo "✅ Integration tests completed!"