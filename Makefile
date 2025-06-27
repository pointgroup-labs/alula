#!/usr/bin/make
WASM_TARGET = wasm32v1-none
WASM_TARGET_DIR = target/$(WASM_TARGET)/release
MOCK_WASM_DIR = wasms

LENDING_CONTRACT := lending

REFLECTOR_ORACLE_URL = https://github.com/reflector-network/reflector-contract/releases/download/v4.1.0_reflector-oracle_v4.1.0.wasm/reflector-oracle_v4.1.0.wasm
REFLECTOR_ORACLE_WASM = $(WASM_TARGET_DIR)/reflector-oracle.wasm
REFLECTOR_ORACLE_MOCK := reflector-oracle-mock

SOROSWAP_ROUTER_URL = https://github.com/soroswap/core/releases/download/workflow%2FsorobanBuildForStellarExpert__contracts_router_soroswap-router_pkg0.0.1_cli21.0.0/soroswap-router_v0.0.1.wasm
SOROSWAP_ROUTER_WASM = $(WASM_TARGET_DIR)/soroswap-router.wasm
SOROSWAP_ROUTER_MOCK := soroswap-router-mock

SOROSWAP_FACTORY_URL = https://github.com/soroswap/core/releases/download/exp%2FgetPreviousHash__contracts_factory_soroswap-factory_pkg0.0.2_cli21.5.0/soroswap-factory_v0.0.2.wasm
SOROSWAP_FACTORY_WASM = $(WASM_TARGET_DIR)/soroswap-factory.wasm

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

clippy: check
	cargo clippy --tests

check: build-init ## Check compilation correctness with cargo
	cargo check --tests

rebuild-lending: build ## Rebuilds lending contract forcefully. Useful when modifying other workspace's contracts
	cargo clean -p $(LENDING_CONTRACT)

# It's important to maintain a valid topological order of the contracts build
build: build-init ## Build contracts
	stellar contract build --package $(REFLECTOR_ORACLE_MOCK)    --out-dir $(MOCK_WASM_DIR)
	stellar contract build --package $(SOROSWAP_ROUTER_MOCK)     --out-dir $(MOCK_WASM_DIR)
	stellar contract build --package $(LENDING_CONTRACT)
	stellar contract build --package $(FLASH_LOAN_TAKER_MOCK)    --out-dir $(MOCK_WASM_DIR)

build-init: ## Build init
	mkdir -p $(WASM_TARGET_DIR)
	mkdir -p $(MOCK_WASM_DIR)
	@echo "Checking for WASM files..."
	$(call download_wasm_contract,$(REFLECTOR_ORACLE_WASM),$(REFLECTOR_ORACLE_URL))
	$(call download_wasm_contract,$(SOROSWAP_ROUTER_WASM),$(SOROSWAP_ROUTER_URL))
	$(call download_wasm_contract,$(SOROSWAP_FACTORY_WASM),$(SOROSWAP_FACTORY_URL))

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
	
test-coverage: ## Test coverage
	cargo +nightly llvm-cov nextest --no-tests=warn --no-report
	cargo +nightly llvm-cov --doc --no-report

fmt: ## Format code using cargo
	cargo fmt --all

clean: ## Clean build artifacts
	cargo clean
	rm -r $(MOCK_WASM_DIR)/*
