all: api_test

api_test:
	rustc --edition=2018 --crate-name api_test  source/api_test.rs --crate-type cdylib --emit=link --target=wasm32-unknown-unknown
	