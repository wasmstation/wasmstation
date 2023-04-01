all:
	cargo build -r --examples --target wasm32-unknown-unknown
	wasm-bindgen --out-dir examples/pkg/ --target web --no-typescript ../target/wasm32-unknown-unknown/release/examples/basic.wasm

serve: all
	python -m http.server -d examples
