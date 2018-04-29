.PHONY: build

build: rust.wasm

rust.wasm: rust.big.wasm
	wasm-gc $< $@

rust.big.wasm: src/lib.rs
	cargo +nightly build --target wasm32-unknown-unknown --release --lib --verbose
	cp target/wasm32-unknown-unknown/release/rust_wasm_test_wavesim.wasm $@
