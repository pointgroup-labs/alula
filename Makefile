#!/usr/bin/make
WASM_TARGET = wasm32v1-none
WASM_TARGET_DIR = target/$(WASM_TARGET)/release

LENDING_CONTRACT := lending
LENDING_CONTRACT_ID := ABC123

REFLECTOR_ORACLE_URL = https://github.com/reflector-network/reflector-contract/releases/download/v4.1.0_reflector-oracle_v4.1.0.wasm/reflector-oracle_v4.1.0.wasm
REFLECTOR_ORACLE_WASM = $(WASM_TARGET_DIR)/reflector_oracle.wasm

REFLECTOR_ORACLE_MOCK := reflector-oracle-mock

FLASH_LOAN_TAKER_MOCK := flash_loan_taker_mock

.DEFAULT_GOAL: help

.PHONY: help
help: ## Show this help
	@printf "\033[33m%s:\033[0m\n" 'Available commands'
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  \033[32m%-18s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

# ----------------------------------------------------------------------------------------------------------------------

check: build-init ## Check compilation correctness with cargo
	cargo check

build: build-init ## Build contracts
	cargo build --release --target $(WASM_TARGET) -p $(LENDING_CONTRACT)
	cargo build --release --target $(WASM_TARGET) -p $(REFLECTOR_ORACLE_MOCK)
	cargo build --release --target $(WASM_TARGET) -p $(FLASH_LOAN_TAKER_MOCK)

build-init: ## Build init
	mkdir -p $(WASM_TARGET_DIR)
	@if [ ! -f $(REFLECTOR_ORACLE_WASM) ]; then \
		echo "Downloading reflector oracle WASM file..."; \
		curl -L $(REFLECTOR_ORACLE_URL) -o $(REFLECTOR_ORACLE_WASM); \
	else \
		echo "Reflector oracle WASM file already exists, skipping download."; \
	fi

build-optimize: build ## Optimize contracts
	mkdir -p target/$(WASM_TARGET)/optimized
	stellar contract optimize \
		--wasm target/$(WASM_TARGET)/release/$(LENDING_CONTRACT).wasm \
		--wasm-out target/$(WASM_TARGET)/optimized/$(LENDING_CONTRACT).wasm
	cd target/$(WASM_TARGET)/optimized/ && \
		for i in *.wasm ; do \
			ls -l "$$i"; \
		done

sdk: build-optimize ## Generate typescript sdk
	stellar contract bindings typescript --overwrite \
		--wasm ./target/$(WASM_TARGET)/optimized/$(LENDING_CONTRACT).wasm --output-dir ./packages/sdk/ \
		--network testnet

test: build ## Run tests
	cargo nextest run --locked --workspace
	#cargo test

test-coverage: ## Test coverage
	cargo +nightly llvm-cov nextest --no-tests=warn --no-report
	cargo +nightly llvm-cov --doc --no-report

fmt: ## Format code using cargo
	cargo fmt --all

clean: ## Clean build artifacts
	cargo clean
