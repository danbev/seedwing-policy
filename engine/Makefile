wit-compile:
	cargo b -p seedwing-policy-engine --target=wasm32-wasi --no-default-features --features=""

wit-component:
	wasm-tools component new -v ../target/wasm32-wasi/debug/seedwing_policy_engine.wasm --adapt wasi_snapshot_preview1.wasm -o seedwing_policy-engine-component.wasm

wit-js-bindings:
	cd js && npm run bindings

wit-js-build:
	cd js && npm run bindings && npm run example

wit-js-run:
	cd js && npm run example
