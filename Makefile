#!/usr/bin/make
WASM_TARGET = wasm32v1-none
WASM_TARGET_DIR = target/$(WASM_TARGET)/release
MOCK_WASM_DIR = wasms

LENDING_CONTRACT := lending

REFLECTOR_ORACLE_URL = https://github.com/reflector-network/reflector-contract/releases/download/v4.1.0_reflector-oracle_v4.1.0.wasm/reflector-oracle_v4.1.0.wasm
REFLECTOR_ORACLE_WASM = $(WASM_TARGET_DIR)/reflector-oracle.wasm
REFLECTOR_ORACLE_MOCK := reflector-oracle-mock

SOROSWAP_AGGREGATOR_URL = https://github.com/soroswap/aggregator/releases/download/feat%2FReleaseForStellarExpert__contracts_aggregator_soroswap-aggregator_pkg1.0.0_cli21.5.0/soroswap-aggregator_v1.0.0.wasm
SOROSWAP_AGGREGATOR_WASM = $(WASM_TARGET_DIR)/soroswap-aggregator.wasm
SOROSWAP_AGGREGATOR_MOCK := soroswap-aggregator-mock

FLASH_LOAN_TAKER_MOCK := flash-loan-taker-mock

.DEFAULT_GOAL: help

.PHONY: help
help: ## Show this help
	@printf "\033[33m%s:\033[0m\n" 'Available commands'
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  \033[32m%-18s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

# Downloads  a WASM file if it doesn't exist
define download_wasm_contract
	@if [ ! -f $(1) ]; then \
		echo "Downloading $(1) WASM file..."; \
		curl -L $(2) -o $(1); \
	else \
		echo "$(1) WASM file already exists, skipping download."; \
	fi
endef

# ----------------------------------------------------------------------------------------------------------------------

check: build-init ## Check compilation correctness with cargo
	cargo check

build: build-init ## Build contracts
	stellar contract build --package $(REFLECTOR_ORACLE_MOCK)    --out-dir $(MOCK_WASM_DIR)
	stellar contract build --package $(FLASH_LOAN_TAKER_MOCK)    --out-dir $(MOCK_WASM_DIR)
	stellar contract build --package $(SOROSWAP_AGGREGATOR_MOCK) --out-dir $(MOCK_WASM_DIR)

	stellar contract build --package $(LENDING_CONTRACT)

build-init: ## Build init
	mkdir -p $(WASM_TARGET_DIR)
	mkdir -p $(MOCK_WASM_DIR)
	@echo "Checking for WASM files..."
	$(call download_wasm_contract,$(REFLECTOR_ORACLE_WASM),$(REFLECTOR_ORACLE_URL))
	$(call download_wasm_contract,$(SOROSWAP_AGGREGATOR_WASM),$(SOROSWAP_AGGREGATOR_URL))

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
	rm -r $(MOCK_WASM_DIR)/*
