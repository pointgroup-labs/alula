#!/usr/bin/make

LENDING_CONTRACT := lending
LENDING_CONTRACT_ID := ABC123

WASM_TARGET_DIR = target/wasm32-unknown-unknown/release
REFLECTOR_ORACLE_URL = https://github.com/reflector-network/reflector-contract/releases/download/v4.1.0_reflector-oracle_v4.1.0.wasm/reflector-oracle_v4.1.0.wasm
REFLECTOR_ORACLE_WASM = $(WASM_TARGET_DIR)/reflector_oracle.wasm

.DEFAULT_GOAL: help

.PHONY: help
help: ## Show this help
	@printf "\033[33m%s:\033[0m\n" 'Available commands'
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  \033[32m%-18s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

# ----------------------------------------------------------------------------------------------------------------------

check: build-init ## Check compilation correctness with cargo
	cargo check

build: build-init ## Build contracts
	stellar contract build

build-init: ## Build init
	mkdir -p $(WASM_TARGET_DIR)
	@if [ ! -f $(REFLECTOR_ORACLE_WASM) ]; then \
		echo "Downloading reflector oracle WASM file..."; \
		curl -L $(REFLECTOR_ORACLE_URL) -o $(REFLECTOR_ORACLE_WASM); \
	else \
		echo "Reflector oracle WASM file already exists, skipping download."; \
	fi

build-optimize: build ## Optimize contracts
	mkdir -p target/wasm32-unknown-unknown/optimized
	stellar contract optimize \
		--wasm target/wasm32-unknown-unknown/release/$(LENDING_CONTRACT).wasm \
		--wasm-out target/wasm32-unknown-unknown/optimized/$(LENDING_CONTRACT).wasm
	cd target/wasm32-unknown-unknown/optimized/ && \
		for i in *.wasm ; do \
			ls -l "$$i"; \
		done

generate-sdk: build-optimize ## Generate typescript sdk
	stellar contract bindings typescript --overwrite \
		--wasm ./target/wasm32-unknown-unknown/optimized/$(LENDING_CONTRACT).wasm --output-dir ./packages/$(LENDING_CONTRACT)-sdk/ \
		--network testnet

test: build ## Run tests
	cargo test

fmt: ## Format code using cargo
	cargo fmt --all

clean: ## Clean build artifacts
	cargo clean
