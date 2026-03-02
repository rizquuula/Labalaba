.PHONY: dev build build-be check install cargo-check test clean help

.DEFAULT_GOAL := dev

# ── Frontend ──────────────────────────────────────────────────────────────────

dev: ## Start full dev environment (FE + BE, hot-reload)
	@echo "Starting backend (daemon)..."
	@cargo run -p labalaba-daemon & \
	cd gui && npm run dev

build: ## Build frontend and backend (release)
	cd gui && npm run build
	cargo build -p labalaba-daemon --release

build-be: ## Build backend only (release)
	cargo build -p labalaba-daemon --release

check: ## Svelte + TypeScript validation
	cd gui && npm run check

install: ## Install frontend npm dependencies
	cd gui && npm install

# ── Rust ──────────────────────────────────────────────────────────────────────

cargo-check: ## Type-check daemon and shared crates
	cargo check -p labalaba-daemon
	cargo check -p labalaba-shared

test: ## Run all Rust tests
	cargo test

# ── Cleanup ───────────────────────────────────────────────────────────────────

clean: ## Remove build artifacts (target/, node_modules/, .svelte-kit/)
	cargo clean
	rm -rf gui/node_modules gui/.svelte-kit gui/build

# ── Help ──────────────────────────────────────────────────────────────────────

help: ## Show available targets
	@grep -E '^[a-zA-Z_-]+:.*##' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*##"}; {printf "  %-16s %s\n", $$1, $$2}'
