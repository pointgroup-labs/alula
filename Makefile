#!/usr/bin/make

# ══════════════════════════════════════════════════════════════════════════════
# Configuration
# ══════════════════════════════════════════════════════════════════════════════

WASM_DIR              := wasms
MOCKS_DIR             := $(WASM_DIR)/mocks
DEPLOY_DIR            := $(WASM_DIR)/deploy
DEPLOY_OPTIMIZED_DIR  := $(WASM_DIR)/deploy_optimized
DOWNLOADS_DIR         := $(WASM_DIR)/downloads

# Contracts
CONTRACTS             := market market_manager aggregated_oracle soroswap_sep_40_adapter
DEPLOY_CONTRACTS      := market market_manager aggregated_oracle soroswap_sep_40_adapter
MOCK_CONTRACTS        := soroswap_router_mock flash_loan_taker_mock

# External dependencies
SOROSWAP_BASE_URL     := https://github.com/soroswap/core/releases/download
SOROSWAP_ROUTER_URL   := $(SOROSWAP_BASE_URL)/workflow%2FsorobanBuildForStellarExpert__contracts_router_soroswap-router_pkg0.0.1_cli21.0.0/soroswap-router_v0.0.1.wasm
SOROSWAP_PAIR_URL     := $(SOROSWAP_BASE_URL)/workflow%2FsorobanBuildForStellarExpert__contracts_pair_soroswap-pair_pkg0.0.1_cli21.0.0/soroswap-pair_v0.0.1.wasm

# Network
NETWORK               := testnet

# Tools
CARGO_NIGHTLY         := cargo +nightly
LLVM_COV              := $(CARGO_NIGHTLY) llvm-cov
LLVM_COV_FLAGS        := --branch

# Metadata
VERSION               := $(shell cargo metadata --format-version 1 --no-deps 2>/dev/null | jq -r '.packages[] | select(.name == "market") | .version' 2>/dev/null || echo "unknown")

# Colors
R := \033[0;31m
G := \033[0;32m
Y := \033[0;33m
B := \033[0;34m
C := \033[0;36m
N := \033[0m

# Make comma (for use in $(call) arguments)
COMMA := ,

.DEFAULT_GOAL := help
.SILENT: help
.PHONY: all build test clean help

# ══════════════════════════════════════════════════════════════════════════════
# Helpers
# ══════════════════════════════════════════════════════════════════════════════

define log
	@printf "$(B)▸$(N) %s\n" $(1)
endef

define success
	@printf "$(G)✓$(N) %s\n" $(1)
endef

define warn
	@printf "$(Y)⚠$(N) %s\n" $(1)
endef

define download_wasm
	@[ -f $(1) ] && printf "$(Y)⊘$(N) %s exists\n" "$(notdir $(1))" || \
		{ printf "$(B)↓$(N) %s\n" "$(notdir $(1))" && curl -fsSL "$(2)" -o "$(1)"; }
endef

define build_contract
	@printf "$(B)⚙$(N) %s\n" "$(1)"
	@stellar contract build --package "$(1)" --out-dir "$(2)" $(3)
endef

define optimize_contract
	@printf "$(B)◎$(N) %s\n" "$(1)"
	@stellar contract optimize --wasm "$(DEPLOY_DIR)/$(1).wasm" --wasm-out "$(DEPLOY_OPTIMIZED_DIR)/$(1).optimized.wasm"
endef

# ══════════════════════════════════════════════════════════════════════════════
# Help
# ══════════════════════════════════════════════════════════════════════════════

help: ## Show this help
	@printf "$(C)jlend$(N) v$(VERSION) · $(G)$(NETWORK)$(N)\n\n"
	@awk 'BEGIN {FS=":.*##"} /^[a-zA-Z0-9_\/.%-]+:.*##/ { \
		gsub(/^[ \t]+|[ \t]+$$/, "", $$1); \
		gsub(/^[ \t]+|[ \t]+$$/, "", $$2); \
		printf "  $(G)%-20s$(N) %s\n", $$1, $$2 \
	}' $(MAKEFILE_LIST)

# ══════════════════════════════════════════════════════════════════════════════
# Setup
# ══════════════════════════════════════════════════════════════════════════════

setup: ## Install development tools
	$(call log,"Installing tools...")
	@cargo install cargo-audit cargo-outdated cargo-nextest cargo-watch cargo-sort cargo-llvm-cov
	$(call success,"Done")

# ══════════════════════════════════════════════════════════════════════════════
# Build
# ══════════════════════════════════════════════════════════════════════════════

build/prepare:
	@mkdir -p $(WASM_DIR) $(MOCKS_DIR) $(DEPLOY_DIR) $(DEPLOY_OPTIMIZED_DIR) $(DOWNLOADS_DIR)
	$(call download_wasm,$(DOWNLOADS_DIR)/soroswap-router.wasm,$(SOROSWAP_ROUTER_URL))
	$(call download_wasm,$(DOWNLOADS_DIR)/soroswap-pair.wasm,$(SOROSWAP_PAIR_URL))

build: build/prepare ## Build all contracts
	$(call build_contract,soroswap_router_mock,$(MOCKS_DIR))
	$(call build_contract,soroswap_sep_40_adapter,$(WASM_DIR))
	$(call build_contract,aggregated_oracle,$(WASM_DIR))
	$(call build_contract,market,$(WASM_DIR))
	$(call build_contract,market_manager,$(WASM_DIR))
	$(call build_contract,flash_loan_taker_mock,$(MOCKS_DIR))
	$(call success,"Build complete")

build/deploy: build/prepare ## Build for deployment
	$(call build_contract,soroswap_sep_40_adapter,$(DEPLOY_DIR))
	$(call build_contract,aggregated_oracle,$(DEPLOY_DIR))
	$(call build_contract,market,$(DEPLOY_DIR),--features deploy)
	$(call build_contract,market_manager,$(DEPLOY_DIR),--features deploy)
	$(call success,"Deploy build complete")

build/mainnet: build/prepare ## Build for mainnet
	$(call build_contract,soroswap_sep_40_adapter,$(DEPLOY_DIR))
	$(call build_contract,aggregated_oracle,$(DEPLOY_DIR))
	$(call build_contract,market,$(DEPLOY_DIR),--features deploy$(COMMA)mainnet)
	$(call build_contract,market_manager,$(DEPLOY_DIR),--features deploy)
	$(call success,"Mainnet build complete")

define do_optimize
	$(call optimize_contract,soroswap_sep_40_adapter)
	$(call optimize_contract,aggregated_oracle)
	$(call optimize_contract,market)
	$(call optimize_contract,market_manager)
	@printf "\n$(C)Sizes:$(N)\n"
	@find $(DEPLOY_OPTIMIZED_DIR) -name "*.wasm" -exec sh -c 'printf "  %-40s %s\n" "$$(basename {})" "$$(ls -lh {} | awk "{print \$$5}")"' \;
endef

build/optimize: build/deploy ## Optimize for deployment
	$(do_optimize)
	$(call success,"Optimization complete")

build/optimize/mainnet: build/mainnet ## Optimize for mainnet
	$(do_optimize)
	$(call success,"Optimization complete (mainnet)")

# ══════════════════════════════════════════════════════════════════════════════
# Testing
# ══════════════════════════════════════════════════════════════════════════════

test: build ## Run tests
	@cargo nextest run --locked --workspace --lib

test/watch: build ## Run tests in watch mode
	@cargo watch -x 'nextest run --workspace --lib'

test/fuzz: build ## Run fuzzing suite
	@RUST_BACKTRACE=1 $(CARGO_NIGHTLY) fuzz run --fuzz-dir=tests/fuzz --sanitizer=none fuzz_target -- -max_len=1048576

# ── Coverage ──────────────────────────────────────────────────────────────────

cov: ## Generate coverage report
	@$(LLVM_COV) nextest $(LLVM_COV_FLAGS) --no-tests=warn --no-report || true
	@$(LLVM_COV) --doc --no-report || true
	@$(LLVM_COV) report $(LLVM_COV_FLAGS)

cov/html: ## Generate HTML coverage report
	@$(LLVM_COV) nextest $(LLVM_COV_FLAGS) --html --no-tests=warn || true
	@$(LLVM_COV) --doc --html || true
	$(call success,"Report: target/llvm-cov/html/index.html")

cov/missing: ## Show uncovered lines
	@$(LLVM_COV) report $(LLVM_COV_FLAGS) --show-missing-lines

cov/lcov: ## Generate LCOV format
	@$(LLVM_COV) nextest $(LLVM_COV_FLAGS) --lcov --output-path target/lcov.info --no-tests=warn || true
	$(call success,"Output: target/lcov.info")

cov/json: ## Generate JSON coverage
	@$(LLVM_COV) nextest $(LLVM_COV_FLAGS) --json --output-path target/coverage.json --no-tests=warn || true
	$(call success,"Output: target/coverage.json")

# ══════════════════════════════════════════════════════════════════════════════
# Code Quality
# ══════════════════════════════════════════════════════════════════════════════

check: ## Run cargo check
	@cargo check --locked --workspace

fmt: ## Format code
	@cargo sort --workspace 2>/dev/null || true
	@$(CARGO_NIGHTLY) fmt --all

fmt/check: ## Check formatting
	@$(CARGO_NIGHTLY) fmt --all --check

lint: check ## Run clippy
	@cargo clippy --workspace --all-targets -- -D warnings

lint/fix: ## Auto-fix lint issues
	@cargo clippy --workspace --all-targets --fix --allow-dirty --allow-staged

# Compound targets
ci:  test lint fmt/check  ## Run full CI pipeline
pre-commit: test lint fmt  ## Pre-commit checks

# ══════════════════════════════════════════════════════════════════════════════
# SDK Generation
# ══════════════════════════════════════════════════════════════════════════════

sdk: build/optimize ## Generate TypeScript SDK
	$(call log,"Generating SDK...")
	@stellar contract bindings typescript --overwrite \
		--wasm "$(DEPLOY_OPTIMIZED_DIR)/market.optimized.wasm" \
		--output-dir ./packages/sdk/market --network "$(NETWORK)"
	@stellar contract bindings typescript --overwrite \
		--wasm "$(DEPLOY_OPTIMIZED_DIR)/market_manager.optimized.wasm" \
		--output-dir ./packages/sdk/market_manager --network "$(NETWORK)"
	$(call success,"SDK generated")

# ══════════════════════════════════════════════════════════════════════════════
# Maintenance
# ══════════════════════════════════════════════════════════════════════════════

outdated: ## Check outdated deps
	@cargo outdated
	@pnpm outdated 2>/dev/null || true

update: ## Update dependencies
	@cargo update
	@pnpm update 2>/dev/null || true

audit: ## Security audit
	@cargo audit
	@pnpm audit 2>/dev/null || true

audit/fix: ## Fix vulnerabilities
	@cargo audit fix || true

size: ## Show WASM sizes
	@printf "$(C)WASM Sizes:$(N)\n"
	@find $(WASM_DIR) -name "*.wasm" -type f | sort | while read f; do \
		printf "  %-45s %s\n" "$$(basename $$f)" "$$(ls -lh "$$f" | awk '{print $$5}')"; \
	done

clean: ## Clean build artifacts
	@cargo clean
	@rm -rf $(WASM_DIR)/* target/llvm-cov 2>/dev/null || true
	$(call success,"Cleaned")

clean/all: clean ## Deep clean (including downloads)
	@rm -rf $(DOWNLOADS_DIR)/* ./packages/sdk/* node_modules 2>/dev/null || true
	$(call success,"Deep cleaned")

# ══════════════════════════════════════════════════════════════════════════════
# Aliases
# ══════════════════════════════════════════════════════════════════════════════

b: build       		## → build
bd: build/deploy    ## → build/deploy
bm: build/mainnet   ## → build/mainnet
t: test        		## → test
c: check       		## → check
l: lint        		## → lint
f: fmt         		## → fmt
w: test/watch  		## → test/watch
