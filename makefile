integration_test:
	cargo test --test '*' -- --test-threads=1 --nocapture
curl:
	curl -X POST https://ngrok.boursenumeriquedafrique.com/mtn -H "Content-Type: application/json" -d '{ "externalId": "d6c83243-c00c-43d6-aa76-bc5c63bf1517", "amount": "100", "currency": "EUR", "payer": {"partyIdType": "MSISDN","partyId": "46733123450"},"payeeNote": "test_payee_note","status": "FAILED","reason": "INTERNAL_PROCESSING_ERROR"}'

push_new_version:
	chmod +x new_version.sh
	./new_version.sh
