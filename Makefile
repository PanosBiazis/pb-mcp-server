CARGO := cargo
CARGO_FLAGS_LIB := --lib
CARGO_FLAGS_RELEASE := --release
CARGO_FLAGS_ALL_FEATURES := --all-features
BIN_SERVER := --bin pb-mcp-server
BIN_GATEWAY := --bin pb-gateway

.PHONY: build build-release test test-all check check-all clippy fmt fmt-check run run-gateway audit clean help

build: ## Build the library
	$(CARGO) build $(CARGO_FLAGS_LIB)

build-release: ## Build the library in release mode
	$(CARGO) build $(CARGO_FLAGS_RELEASE) $(CARGO_FLAGS_LIB)

test: ## Run tests with default features
	$(CARGO) test $(CARGO_FLAGS_LIB)

test-all: ## Run tests with all features
	$(CARGO) test $(CARGO_FLAGS_LIB) $(CARGO_FLAGS_ALL_FEATURES)

check: ## Check the library with default features
	$(CARGO) check $(CARGO_FLAGS_LIB)

check-all: ## Check the library with all features
	$(CARGO) check $(CARGO_FLAGS_LIB) $(CARGO_FLAGS_ALL_FEATURES)

clippy: ## Run clippy with warnings as errors
	$(CARGO) clippy $(CARGO_FLAGS_LIB) -- -D warnings

fmt: ## Format code
	$(CARGO) fmt

fmt-check: ## Check code formatting
	$(CARGO) fmt --check

run: ## Run the pb-mcp-server binary
	$(CARGO) run $(BIN_SERVER)

run-gateway: ## Run the pb-gateway binary
	$(CARGO) run $(BIN_GATEWAY)

audit: ## Audit dependencies for vulnerabilities
	$(CARGO) audit

clean: ## Remove build artifacts
	$(CARGO) clean

help: ## Show this help message
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'
