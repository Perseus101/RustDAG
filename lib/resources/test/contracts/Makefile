all: full_api_test raw_api_test

full_api_test:
	rustc --edition=2018 --crate-name full_api_test  source/full_api_test.rs --crate-type cdylib --emit=link --target=wasm32-unknown-unknown

raw_api_test:
	rustc --edition=2018 --crate-name raw_api_test  source/raw_api_test.rs --crate-type cdylib --emit=link --target=wasm32-unknown-unknown