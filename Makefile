.PHONY: dev stop release-windows release-linux release-mac build-be sidecar sidecar-dev check install cargo-check test clean tree version help

.DEFAULT_GOAL := help

VERSION     := $(shell node -e "const fs=require('fs');console.log(JSON.parse(fs.readFileSync('gui/src-tauri/tauri.conf.json')).version)")
TARGET_DIR  := $(if $(CARGO_TARGET_DIR),$(CARGO_TARGET_DIR),$(CURDIR)/target)
HOST_TRIPLE := $(shell rustc -vV | sed -n 's/host: //p')
EXE         := $(if $(findstring windows,$(HOST_TRIPLE)),.exe,)

# - Dev ---------------------------------

dev: sidecar-dev ## Start Tauri app (daemon + GUI with hot-reload)
	cd gui && LABALABA_DATA_DIR="$(CURDIR)" LABALABA_DAEMON_BIN="$(TARGET_DIR)/debug/labalaba-daemon$(EXE)" npm run tauri dev

stop: ## Kill all dev processes (daemon + Tauri)
	@powershell -ExecutionPolicy Bypass -File scripts/stop.ps1

# NOTE: release-windows requires gui/src-tauri/binaries/labalaba-daemon-<triple>.exe staged first.
# CI handles staging via the "Build daemon sidecar" step in release.yml.
release-windows: ## Build Tauri release for Windows → dist/labalaba-v$(VERSION).exe
	cd gui && npm run tauri build
	@mkdir -p dist
	cp gui/src-tauri/target/release/labalaba-gui.exe dist/labalaba-v$(VERSION).exe
	@echo "Output: dist/labalaba-v$(VERSION).exe"

release-linux: sidecar ## Build Tauri release for Linux → dist/labalaba-v$(VERSION)
	cd gui && npm run tauri build
	@mkdir -p dist
	cp gui/src-tauri/target/release/labalaba-gui dist/labalaba-v$(VERSION)
	@echo "Output: dist/labalaba-v$(VERSION)"

release-mac: sidecar ## Build Tauri release for macOS → dist/labalaba-v$(VERSION).app
	cd gui && npm run tauri build
	@mkdir -p dist
	cp -r gui/src-tauri/target/release/bundle/macos/labalaba-gui.app dist/labalaba-v$(VERSION).app
	@echo "Output: dist/labalaba-v$(VERSION).app"

build-be: ## Build backend only (release)
	cargo build -p labalaba-daemon --release

sidecar: ## Build the daemon and stage it as a Tauri sidecar (release, host target)
	cargo build -p labalaba-daemon --release
	@mkdir -p gui/src-tauri/binaries
	cp "$(TARGET_DIR)/release/labalaba-daemon$(EXE)" "gui/src-tauri/binaries/labalaba-daemon-$(HOST_TRIPLE)$(EXE)"

sidecar-dev: ## Build the daemon (debug) and stage it as a sidecar for `tauri dev`
	cargo build -p labalaba-daemon
	@mkdir -p gui/src-tauri/binaries
	cp "$(TARGET_DIR)/debug/labalaba-daemon$(EXE)" "gui/src-tauri/binaries/labalaba-daemon-$(HOST_TRIPLE)$(EXE)"

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

# - Info -----------------------------------

tree: ## Print project tree with per-file line counts
	@bash scripts/tree.sh

version: ## Print the current project version
	@echo $(VERSION)

# - Help -----------------------------------

help: ## Show available targets
	@grep -E '^[a-zA-Z_-]+:.*##' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*##"}; {printf "  %-16s %s\n", $$1, $$2}'
