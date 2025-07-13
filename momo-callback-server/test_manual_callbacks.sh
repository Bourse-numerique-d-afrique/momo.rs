#!/bin/bash

# Manual callback testing script
# This simulates MTN MoMo sending callbacks to your server

SERVER_URL="https://localhost"  # Change to your server URL

echo "🧪 Testing MTN MoMo Callback Server Endpoints"
echo "Server URL: $SERVER_URL"

# Test health check
echo -e "\n📊 Testing health check..."
curl -k -X GET "$SERVER_URL/health"

# Test successful payment callback
echo -e "\n\n💰 Testing successful payment callback..."
curl -k -X POST "$SERVER_URL/collection_request_to_pay/REQUEST_TO_PAY" \
  -H "Content-Type: application/json" \
  -d '{
    "financialTransactionId": "12345678",
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

# Test failed payment callback
echo -e "\n\n❌ Testing failed payment callback..."
curl -k -X POST "$SERVER_URL/collection_request_to_pay/REQUEST_TO_PAY" \
  -H "Content-Type: application/json" \
  -d '{
    "financialTransactionId": "12345679",
    "externalId": "test-payment-002",
    "amount": "50",
    "currency": "UGX",
    "payer": {
      "partyIdType": "MSISDN",
      "partyId": "+256123456789"
    },
    "payeeNote": "Test failed payment",
    "payerMessage": "Test message",
    "status": "FAILED",
    "reason": {
      "code": "PAYER_NOT_FOUND",
      "message": "Payer not found"
    }
  }'

# Test invoice callback
echo -e "\n\n🧾 Testing invoice callback..."
curl -k -X POST "$SERVER_URL/collection_invoice/INVOICE" \
  -H "Content-Type: application/json" \
  -d '{
    "referenceId": "invoice-ref-001",
    "externalId": "invoice-ext-001",
    "amount": "200",
    "currency": "UGX",
    "status": "SUCCESSFUL",
    "paymentReference": "payment-ref-001",
    "invoiceId": "invoice-001",
    "expiryDateTime": "2024-12-31T23:59:59Z",
    "intendedPayer": {
      "partyIdType": "MSISDN",
      "partyId": "+256123456789"
    },
    "description": "Test invoice payment"
  }'

# Test disbursement callback
echo -e "\n\n💸 Testing disbursement callback..."
curl -k -X POST "$SERVER_URL/disbursement_deposit_v1/DISBURSEMENT_DEPOSIT_V1" \
  -H "Content-Type: application/json" \
  -d '{
    "referenceId": "disbursement-ref-001",
    "status": "SUCCESSFUL",
    "financialTransactionId": "disbursement-tx-001"
  }'

echo -e "\n\n✅ Testing complete! Check your server logs for callback processing."