# Copyright (c) 2025 Erick Bourgeois, firestoned
# SPDX-License-Identifier: MIT

.PHONY: help fmt clippy test test-expand doc check clean build sbom sbom-validate audit security-check

help: ## Show this help message
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

fmt: ## Format all code
	@echo "Formatting code..."
	@cargo fmt --all

clippy: ## Run clippy on all crates
	@echo "Running clippy..."
	@cargo clippy --all -- -D warnings -W clippy::pedantic -A clippy::module_name_repetitions

test: ## Run all tests
	@echo "Running tests..."
	@cargo test --all

test-expand: ## Verify macro expansion
	@echo "Verifying macro expansion..."
	@cargo expand --lib kube-condition-derive

doc: ## Build documentation
	@echo "Building documentation..."
	@cargo doc --all --no-deps --open

check: fmt clippy test ## Run all checks (fmt, clippy, test)
	@echo "All checks passed!"

build: ## Build all crates
	@echo "Building all crates..."
	@cargo build --all

clean: ## Clean build artifacts
	@echo "Cleaning build artifacts..."
	@cargo clean

sbom: ## Generate SBOM for all crates (CycloneDX JSON)
	@echo "Generating SBOM for kube-condition..."
	@cargo cyclonedx --all --format json --describe binaries
	@echo "✓ SBOM files generated: *.cdx.json"

sbom-validate: ## Validate SBOM files
	@echo "Validating SBOM files..."
	@command -v cyclonedx >/dev/null 2>&1 || { echo "ERROR: cyclonedx-cli not installed. Install with: npm install -g @cyclonedx/cyclonedx-cli"; exit 1; }
	@for file in *.cdx.json; do \
		if [ -f "$$file" ]; then \
			echo "Validating $$file..."; \
			cyclonedx validate --input-file "$$file" --fail-on-errors; \
		fi \
	done
	@echo "✓ All SBOM files are valid"

audit: ## Run security audit on dependencies
	@echo "Running cargo audit..."
	@cargo audit

security-check: audit ## Complete security check (audit + vulnerability scan)
	@echo "Running vulnerability scan..."
	@cargo audit --json > audit-report.json || true
	@if [ -f audit-report.json ]; then \
		echo "✓ Audit report generated: audit-report.json"; \
	fi
	@echo "✓ Security checks complete"

.DEFAULT_GOAL := help
