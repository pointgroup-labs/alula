WASM_DIR := wasms
MOCKS_DIR := $(WASM_DIR)/mocks
DEPLOY_DIR := $(WASM_DIR)/deploy
OPTIMIZED_DIR := $(WASM_DIR)/optimized
DOWNLOADS_DIR := $(WASM_DIR)/downloads

LENDING_CONTRACT := lending

REFLECTOR_ORACLE_URL := https://github.com/reflector-network/reflector-contract/releases/download/v4.1.0_reflector-oracle_v4.1.0.wasm/reflector-oracle_v4.1.0.wasm
REFLECTOR_ORACLE_WASM := $(DOWNLOADS_DIR)/reflector-oracle.wasm
REFLECTOR_ORACLE_MOCK := reflector-oracle-mock

SOROSWAP_ROUTER_URL := https://github.com/soroswap/core/releases/download/workflow%2FsorobanBuildForStellarExpert__contracts_router_soroswap-router_pkg0.0.1_cli21.0.0/soroswap-router_v0.0.1.wasm
SOROSWAP_ROUTER_WASM := $(DOWNLOADS_DIR)/soroswap-router.wasm
SOROSWAP_ROUTER_MOCK := soroswap-router-mock

FLASH_LOAN_TAKER_MOCK := flash-loan-taker-mock

.DEFAULT_GOAL: help
.PHONY: help
	
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

help: ## Show this help
	@printf "\033[33m%s:\033[0m\n" 'Available commands'
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  \033[32m%-18s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

clippy: check ## Check common mistakes with clippy
	cargo clippy --tests

check: build ## Check compilation correctness with cargo
	cargo check --tests

# It's important to maintain a valid topological order of the contracts
build: build-init ## Build contracts
	stellar contract build --package $(REFLECTOR_ORACLE_MOCK)    --out-dir $(MOCKS_DIR)
	stellar contract build --package $(SOROSWAP_ROUTER_MOCK)     --out-dir $(MOCKS_DIR)
	stellar contract build --package $(LENDING_CONTRACT)		 --out-dir $(WASM_DIR)
	stellar contract build --package $(FLASH_LOAN_TAKER_MOCK)    --out-dir $(MOCKS_DIR)

build-sdk: download ## Build contracts for deployment
	stellar contract build --package $(LENDING_CONTRACT) --out-dir $(DEPLOY_DIR) --features deploy

build-init: ## Build init
	mkdir -p $(WASM_DIR)
	mkdir -p $(MOCKS_DIR)
	mkdir -p $(DEPLOY_DIR)
	mkdir -p $(DOWNLOADS_DIR)


download: build-init ## Downloads dependency contracts
	@echo "Checking for WASM files..."
	$(call download_wasm_contract,$(REFLECTOR_ORACLE_WASM),$(REFLECTOR_ORACLE_URL))
	$(call download_wasm_contract,$(SOROSWAP_ROUTER_WASM),$(SOROSWAP_ROUTER_URL))

build-optimize: build-sdk ## Optimize contracts
	mkdir -p $(OPTIMIZED_DIR)
	stellar contract optimize \
		--wasm $(DEPLOY_DIR)/$(LENDING_CONTRACT).wasm \
		--wasm-out $(OPTIMIZED_DIR)/$(LENDING_CONTRACT).wasm

sdk: build-optimize ## Generate typescript sdk
	stellar contract bindings typescript --overwrite \
		--wasm $(OPTIMIZED_DIR)/$(LENDING_CONTRACT).wasm --output-dir ./packages/sdk/ \
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
	rm -r $(WASM_DIR)/*
