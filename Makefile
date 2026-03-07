.PHONY: dev stop release-windows release-linux release-mac build-be check install cargo-check test clean help

.DEFAULT_GOAL := help

VERSION := $(shell node -e "const fs=require('fs');console.log(JSON.parse(fs.readFileSync('gui/src-tauri/tauri.conf.json')).version)")

# - Dev ---------------------------------

dev: ## Start Tauri app (daemon + GUI with hot-reload)
	cd gui && LABALABA_DATA_DIR="$(CURDIR)" npm run tauri dev

stop: ## Kill all dev processes (daemon + Tauri)
	@powershell -ExecutionPolicy Bypass -File scripts/stop.ps1

release-windows: ## Build Tauri release for Windows → dist/labalaba-v$(VERSION).exe
	cd gui && npm run tauri build
	@mkdir -p dist
	cp gui/src-tauri/target/release/labalaba-gui.exe dist/labalaba-v$(VERSION).exe
	@echo "Output: dist/labalaba-v$(VERSION).exe"

release-linux: ## Build Tauri release for Linux → dist/labalaba-v$(VERSION)
	cd gui && npm run tauri build
	@mkdir -p dist
	cp gui/src-tauri/target/release/labalaba-gui dist/labalaba-v$(VERSION)
	@echo "Output: dist/labalaba-v$(VERSION)"

release-mac: ## Build Tauri release for macOS → dist/labalaba-v$(VERSION).app
	cd gui && npm run tauri build
	@mkdir -p dist
	cp -r gui/src-tauri/target/release/bundle/macos/labalaba-gui.app dist/labalaba-v$(VERSION).app
	@echo "Output: dist/labalaba-v$(VERSION).app"

build-be: ## Build backend only (release)
	cargo build -p labalaba-daemon --release

check: ## Svelte + TypeScript validation
	cd gui && npm run check

install: ## Install frontend npm dependencies
	cd gui && npm install

# - Rust -----------------------------------

cargo-check: ## Type-check daemon and shared crates
	cargo check -p labalaba-daemon
	cargo check -p labalaba-shared

test: ## Run all Rust tests
	cargo test

# - Cleanup ---------------------------------─

clean: ## Remove build artifacts (target/, node_modules/, .svelte-kit/)
	cargo clean
	rm -rf gui/node_modules gui/.svelte-kit gui/build

# - Help -----------------------------------

help: ## Show available targets
	@grep -E '^[a-zA-Z_-]+:.*##' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*##"}; {printf "  %-16s %s\n", $$1, $$2}'
