.PHONY: dev build check install cargo-check test run-daemon clean help

.DEFAULT_GOAL := dev

# ── Frontend ──────────────────────────────────────────────────────────────────

dev: ## Start full dev environment (GUI + daemon, hot-reload)
	cd gui && npm run tauri dev

build: ## Production release build (MSI / AppImage / DMG)
	cd gui && npm run tauri build

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

run-daemon: ## Start daemon standalone (debug mode)
	cargo run -p labalaba-daemon

# ── Cleanup ───────────────────────────────────────────────────────────────────

clean: ## Remove build artifacts (target/, node_modules/, .svelte-kit/)
	cargo clean
	rm -rf gui/node_modules gui/.svelte-kit gui/build

# ── Help ──────────────────────────────────────────────────────────────────────

help: ## Show available targets
	@grep -E '^[a-zA-Z_-]+:.*##' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*##"}; {printf "  %-16s %s\n", $$1, $$2}'
