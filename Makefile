.PHONY: check fmt clippy test audit deny bench coverage build doc clean

check: fmt clippy test audit deny

fmt:
	cargo fmt --check

clippy:
	cargo clippy --all-features --all-targets -- -D warnings

test:
	cargo test --all-features
	cargo test --no-default-features --features std

audit:
	cargo audit

deny:
	cargo deny check

bench:
	cargo bench

coverage:
	cargo llvm-cov --all-features --lcov --output-path lcov.info

build:
	cargo build --release --all-features

doc:
	RUSTDOCFLAGS="-D warnings" cargo doc --all-features --no-deps

clean:
	cargo clean
	rm -rf coverage/ lcov.info
