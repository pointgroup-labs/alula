#!/usr/bin/make

# ---- Definitions ----
# -- WASM Directories --

WASM_DIR := wasms
MOCKS_DIR := $(WASM_DIR)/mocks
DEPLOY_DIR := $(WASM_DIR)/deploy
DEPLOY_OPTIMIZED_DIR := $(WASM_DIR)/deploy_optimized
DOWNLOADS_DIR := $(WASM_DIR)/downloads

# -- Repository Contracts --

MARKET_MANAGER_CONTRACT := market_manager
MARKET_CONTRACT := market

AGGREGATED_ORACLE_CONTRACT := aggregated_oracle
SOROSWAP_SEP_40_ADAPTER_CONTRACT := soroswap_sep_40_adapter

FLASH_LOAN_TAKER_MOCK_CONTRACT := flash_loan_taker_mock

# -- Dependency Contracts --

SOROSWAP_ROUTER_URL := https://github.com/soroswap/core/releases/download/workflow%2FsorobanBuildForStellarExpert__contracts_router_soroswap-router_pkg0.0.1_cli21.0.0/soroswap-router_v0.0.1.wasm
SOROSWAP_ROUTER_WASM := $(DOWNLOADS_DIR)/soroswap-router.wasm

SOROSWAP_ROUTER_MOCK := soroswap_router_mock

SOROSWAP_PAIR_URL := https://github.com/soroswap/core/releases/download/workflow%2FsorobanBuildForStellarExpert__contracts_pair_soroswap-pair_pkg0.0.1_cli21.0.0/soroswap-pair_v0.0.1.wasm
SOROSWAP_PAIR_WASM := $(DOWNLOADS_DIR)/soroswap-pair.wasm

# -- Network Configuration --

NETWORK := testnet
RPC_URL := https://soroban-testnet.stellar.org:443

# -- Optional tools (installed in setup) -- 

OPTIONAL_TOOLS := cargo-audit cargo-outdated cargo-nextest cargo-watch cargo-sort cargo-llvm-cov

# -- Derived metadata (requires jq, checked in check/tools) --

PACKAGE_VERSION := $(shell cargo metadata --format-version 1 --no-deps | jq -r '.packages[] | select(.name == "$(MARKET_CONTRACT)") | .version')

# -- Colors --

RED     := \033[0;31m
GREEN   := \033[0;32m
YELLOW  := \033[0;33m
BLUE    := \033[0;34m
CYAN    := \033[0;36m
NC      := \033[0m

.DEFAULT_GOAL := help

# -- Helpers --

# Verifies that the required tool exists in the system
define require_tool
	@command -v $(1) >/dev/null 2>&1 || { \
		printf "$(RED)Error: Required tool '$(1)' is not installed$(NC)\n"; exit 1; \
	}
endef

# Downloads a WASM file if it doesn't exist
define download_wasm_contract
	@if [ ! -f $(1) ]; then \
		echo "$(BLUE)Downloading $(1) WASM file...$(NC)"; \
		curl -fsSL --progress-bar "$(2)" -o "$(1)" || { \
			printf "$(RED)Failed to download '$(1)'$(NC)\n"; exit 1; }; \
	else \
		echo "$(YELLOW) WASM file '$(1)' already exists, skipping download...$(NC)"; \
	fi
endef

# Build a contract
define build_contract
	@echo "$(BLUE)Building $(1)...$(NC)"
	stellar contract build --package "$(1)" --out-dir "$(2)" $(3)
endef

# ---- Targets ----

help: ## Show this help
	@printf "$(YELLOW)%s:$(NC)\n" 'Available commands'
	@awk 'BEGIN {FS=":.*##"} /^[[:alnum:]_\/.%\-]+:.*##/ {name=$$1; desc=$$2; gsub(/^[[:space:]]+|[[:space:]]+$$/,"",name); gsub(/^[[:space:]]+|[[:space:]]+$$/,"",desc); printf "  $(GREEN)%-22s$(NC) %s\n", name, desc}' $(MAKEFILE_LIST)
	@printf "\n$(CYAN)Configuration:$(NC)\n"
	@printf "  Version: $(GREEN)$(PACKAGE_VERSION)$(NC)\n"
	@printf "  Network: $(GREEN)$(NETWORK)$(NC)\n"

# ----------------------------------------------------------------------------------------------------------------------
# Development Environment
# ----------------------------------------------------------------------------------------------------------------------

install-dependencies: ## Initialize project
	@echo "$(BLUE)Setting up project...$(NC)"
	cargo install $(OPTIONAL_TOOLS)
	# pnpm install --frozen-lockfile

# ----------------------------------------------------------------------------------------------------------------------
# Build Targets
# ----------------------------------------------------------------------------------------------------------------------


build/prepare: ## Download dependency WASMs
	@mkdir -p $(WASM_DIR) $(MOCKS_DIR) $(DEPLOY_DIR) $(DEPLOY_OPTIMIZED_DIR) $(DOWNLOADS_DIR)
	$(call download_wasm_contract,$(SOROSWAP_ROUTER_WASM),$(SOROSWAP_ROUTER_URL))
	$(call download_wasm_contract,$(SOROSWAP_PAIR_WASM),$(SOROSWAP_PAIR_URL))


# Must maintain the topological order of the building process
build: build/prepare ## Build contracts
	$(call build_contract,$(SOROSWAP_ROUTER_MOCK),$(MOCKS_DIR))

	$(call build_contract,$(SOROSWAP_SEP_40_ADAPTER_CONTRACT),$(WASM_DIR))
	$(call build_contract,$(AGGREGATED_ORACLE_CONTRACT),$(WASM_DIR))

	$(call build_contract,$(MARKET_CONTRACT),$(WASM_DIR))
	$(call build_contract,$(MARKET_MANAGER_CONTRACT),$(WASM_DIR))

	$(call build_contract,$(FLASH_LOAN_TAKER_MOCK_CONTRACT),$(MOCKS_DIR))

build/deploy: build/prepare ## Build contracts for deployment
	$(call build_contract,$(SOROSWAP_SEP_40_ADAPTER_CONTRACT),$(DEPLOY_DIR))
	$(call build_contract,$(AGGREGATED_ORACLE_CONTRACT),$(DEPLOY_DIR))

	$(call build_contract,$(MARKET_CONTRACT),$(DEPLOY_DIR),--features deploy)
	$(call build_contract,$(MARKET_MANAGER_CONTRACT),$(DEPLOY_DIR),--features deploy)

build/optimize: build/deploy ## Optimize contracts
	@echo "$(BLUE)Optimizing contracts...$(NC)"

	@stellar contract optimize \
		--wasm "$(DEPLOY_DIR)/$(SOROSWAP_SEP_40_ADAPTER_CONTRACT).wasm" \
		--wasm-out $(WASM_DIR)/deploy_optimized/$(SOROSWAP_SEP_40_ADAPTER_CONTRACT).optimized.wasm
	@stellar contract optimize \
		--wasm "$(DEPLOY_DIR)/$(AGGREGATED_ORACLE_CONTRACT).wasm" \
		--wasm-out $(WASM_DIR)/deploy_optimized/$(AGGREGATED_ORACLE_CONTRACT).optimized.wasm
	
	@stellar contract optimize \
		--wasm "$(DEPLOY_DIR)/$(MARKET_CONTRACT).wasm" \
		--wasm-out $(WASM_DIR)/deploy_optimized/$(MARKET_CONTRACT).optimized.wasm
	@stellar contract optimize \
		--wasm "$(DEPLOY_DIR)/$(MARKET_MANAGER_CONTRACT).wasm" \
		--wasm-out $(WASM_DIR)/deploy_optimized/$(MARKET_MANAGER_CONTRACT).optimized.wasm

	@ls -lh "$(DEPLOY_OPTIMIZED_DIR)/$(SOROSWAP_SEP_40_ADAPTER_CONTRACT).optimized.wasm" 2>/dev/null || true
	@ls -lh "$(DEPLOY_OPTIMIZED_DIR)/$(AGGREGATED_ORACLE_CONTRACT).optimized.wasm" 2>/dev/null || true

	@ls -lh "$(DEPLOY_OPTIMIZED_DIR)/$(MARKET_CONTRACT).optimized.wasm" 2>/dev/null || true
	@ls -lh "$(DEPLOY_OPTIMIZED_DIR)/$(MARKET_MANAGER_CONTRACT).optimized.wasm" 2>/dev/null || true



# ----------------------------------------------------------------------------------------------------------------------
# Testing
# ----------------------------------------------------------------------------------------------------------------------

test: build
	@echo "$(BLUE)Running unit tests...$(NC)"
	@cargo nextest run --locked --workspace --lib

test/fuzz: build ## Run fuzzing suite
	RUST_BACKTRACE=1 cargo +nightly fuzz run --fuzz-dir=tests/fuzz --sanitizer=none fuzz_target -- -max_len=1048576

test/coverage: ## Generate test coverage
	@cargo +nightly llvm-cov nextest --no-tests=warn --no-report || true
	@cargo +nightly llvm-cov --doc --no-report || true

test/coverage/missing: ## Show missing test coverage lines
	cargo +nightly llvm-cov report --show-missing-lines || true

test/coverage/html: ## Generate HTML test coverage report
	@cargo +nightly llvm-cov nextest --html --no-tests=warn || true
	@cargo +nightly llvm-cov --doc --html || true
	@echo "HTML coverage: target/llvm-cov/html/"


# ----------------------------------------------------------------------------------------------------------------------
# Code Generation
# ----------------------------------------------------------------------------------------------------------------------

sdk: build/optimize ## Generate TypeScript SDK
	@echo "$(BLUE)Generating TypeScript SDK...$(NC)"
	@stellar contract bindings typescript --overwrite \
		--wasm "$(DEPLOY_OPTIMIZED_DIR)/$(MARKET_CONTRACT).optimized.wasm" \
		--output-dir ./packages/sdk/$(MARKET_CONTRACT) \
		--network "$(NETWORK)"
	@stellar contract bindings typescript --overwrite \
		--wasm "$(DEPLOY_OPTIMIZED_DIR)/$(MARKET_MANAGER_CONTRACT).optimized.wasm" \
		--output-dir ./packages/sdk/$(MARKET_MANAGER_CONTRACT) \
		--network "$(NETWORK)"

# ----------------------------------------------------------------------------------------------------------------------
# Code Quality
# ----------------------------------------------------------------------------------------------------------------------

check: build ## Run cargo check
	@echo "$(BLUE)Running cargo check...$(NC)"
	cargo check --locked --workspace

fmt: ## Format and organize code
	@echo "$(BLUE)Formatting code...$(NC)"
	cargo sort --workspace || true
	cargo +nightly fmt --all
	# pnpm lint:fix

fmt/check: ## Check code formatting
	cargo +nightly fmt --all --check

clippy: check ## Run clippy lints
	@echo "$(BLUE)Running clippy...$(NC)"
	cargo clippy --workspace --all-targets -- -D warnings

clippy/fix: ## Auto-fix clippy issues
	cargo clippy --workspace --all-targets --fix --allow-dirty --allow-staged

# ----------------------------------------------------------------------------------------------------------------------
# Maintenance
# ----------------------------------------------------------------------------------------------------------------------

outdated: ## List outdated dependencies
	command -v cargo-outdated >/dev/null 2>&1 || cargo install cargo-outdated
	cargo outdated
	pnpm outdated

update: ## Update dependencies
	@echo "$(BLUE)Updating dependencies...$(NC)"
	cargo update
	pnpm update

size: ## Analyze WASM file sizes
	@printf "$(CYAN)WASM File Sizes:$(NC)\n"
	@printf "$(BLUE)%-30s %10s$(NC)\n" "File" "Size"
	@printf "$(BLUE)%-30s %10s$(NC)\n" "----" "----"
	@find "$(WASM_DIR)" -name "*.wasm" -type f | while read -r file; do \
		size=$$(wc -c < "$$file" | tr -d ' '); \
		human_size=$$(numfmt --to=iec-i --suffix=B $$size 2>/dev/null || echo "$$size B"); \
		printf "$(GREEN)%-30s %10s$(NC)\n" "$$(basename "$$file")" "$$human_size"; \
	done

audit: ## Audit dependencies
	@echo "$(BLUE)Audit dependencies...$(NC)"
	@$(call require_tool,cargo-audit)
	cargo audit
	pnpm audit

clean: ## Clean build artifacts
	cargo clean
	rm -rf "$(WASM_DIR)"/* 2>/dev/null || true
	rm -rf target/llvm-cov 2>/dev/null || true

clean/all: clean ## Clean all artifacts including downloads
	rm -rf "$(DOWNLOADS_DIR)"/* 2>/dev/null || true
	rm -rf ./packages/sdk/* 2>/dev/null || true
	rm -rf node_modules 2>/dev/null || true
