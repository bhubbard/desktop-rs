.PHONY: all build test check fmt clippy coverage clean install

all: build

build:
	cargo build --release

test:
	cargo test --workspace --all-targets

check:
	cargo check --workspace --all-targets

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

clippy:
	cargo clippy --workspace --all-targets -- -D warnings

coverage:
	cargo llvm-cov --workspace --html --output-dir ./coverage_report
	@echo "Coverage HTML report generated at ./coverage_report/html/index.html"

install:
	cargo install --path crates/desktop-cli

clean:
	cargo clean
	rm -rf coverage_report target/coverage lcov.info
