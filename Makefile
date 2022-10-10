.PHONY: all build check clean docs format lint test \
        test.debug test.headless test.headless.debug itest.unit test.unit.debug

all: build

build:
	@cargo build

build.release:
	@cargo build --release

check:
	@cargo check

clean:
	@cargo clean

docs:
	@cargo doc --open --document-private-items

format:
	cargo +nightly fmt

lint:
	@cargo clippy

test:
	@cargo test --all

test.debug:
	@RUST_BACKTRACE=$(RUST_BACKTRACE) cargo test --all -- --nocapture

test.headless:
	@CHROMEDRIVER=$(CHROMEDRIVER) cargo test --target wasm32-unknown-unknown

test.headless.debug:
	@RUST_BACKTRACE=$(RUST_BACKTRACE) CHROMEDRIVER=$(CHROMEDRIVER) cargo test --target wasm32-unknown-unknown

test.unit:
	@cargo test --lib

test.unit.debug:
	@RUST_BACKTRACE=$(RUST_BACKTRACE) cargo test --lib -- --nocapture
