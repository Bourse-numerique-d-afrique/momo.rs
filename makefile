integration_test:
	cargo test --test '*' -- --test-threads=1 --nocapture
curl:
	curl -X POST https://ngrok.boursenumeriquedafrique.com/mtn -H "Content-Type: application/json" -d '{"key":"value"}'

push_new_version:
	chmod +x new_version.sh
	./new_version.sh
