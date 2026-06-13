# Installation

Get Labalaba running on your computer — download a pre-built package or build from source.

---

## System requirements

| Platform | Minimum requirement |
|---|---|
| Windows | Windows 10 or later, x64 |
| Linux | x64, AppImage-compatible distribution (glibc 2.17+) |
| macOS | macOS 11 (Big Sur) or later — Intel or Apple Silicon |

---

## Download a pre-built release

All releases are published on the [Labalaba GitHub Releases page](https://github.com/rizquuula/labalaba/releases).

| Platform | File type | Notes |
|---|---|---|
| Windows x64 | `.msi` installer | Run the installer; no extra tools needed |
| Linux x64 | `.AppImage` | Single portable file; no installation required |
| macOS Intel | `.dmg` | Drag to Applications as usual |
| macOS Apple Silicon (M1/M2/M3) | `.dmg` | Separate build, optimised for ARM |

---

## Install by platform

### Windows

1. Download the `.msi` file for your version.
2. Double-click the file and follow the installer prompts.
3. Launch **Labalaba** from the Start menu or your desktop shortcut.

> **Note:** Windows may show a SmartScreen warning for unsigned installers. Click **More info** then **Run anyway** to proceed.

### Linux

1. Download the `.AppImage` file.
2. Make it executable:
   ```bash
   chmod +x Labalaba_*.AppImage
   ```
3. Run it:
   ```bash
   ./Labalaba_*.AppImage
   ```

> **Tip:** Move the AppImage to `~/Applications/` (or any folder on your `$PATH`) and create a desktop shortcut for easier access.

### macOS

1. Download the `.dmg` file that matches your chip (Intel or Apple Silicon).
2. Open the `.dmg` and drag **Labalaba** into your **Applications** folder.
3. Open Labalaba from **Applications** or Spotlight.

> **Note:** On first launch, macOS may say the app "cannot be opened because the developer cannot be verified." Go to **System Settings → Privacy & Security** and click **Open Anyway**.

---

## Build from source

Building from source lets you run the latest development version or create a distribution package yourself.

### Prerequisites

| Tool | Minimum version |
|---|---|
| Rust (with Cargo) | 1.75 or later |
| Node.js | 18 or later |
| npm | bundled with Node.js |

### Steps

1. Clone the repository:
   ```bash
   git clone https://github.com/rizquuula/labalaba.git
   cd labalaba
   ```

2. Install frontend dependencies:
   ```bash
   make install
   ```

3. Start the app in development mode (hot-reload):
   ```bash
   make dev
   ```

4. To build a distributable installer/package for your current OS:
   ```bash
   cd gui && npm run tauri build
   ```
   The finished bundles are written to:
   ```
   gui/src-tauri/target/release/bundle/
   ```

---

## First launch

When you open Labalaba for the first time, the background engine starts **automatically inside the app** — there is nothing else to install, configure, or run separately. The main window appears ready to use.

If you see **"Connecting to daemon…"** briefly at startup, that is normal; it disappears once the engine is ready (usually under a second).

---

## Where your data is stored

Labalaba keeps its data files in the app's working directory by default (next to the binary in production):

| File | Contents |
|---|---|
| `tasks.yaml` | All your saved tasks |
| `settings.yaml` | App settings |
| `logs/` | Per-task log files |

You can move this location by setting the `LABALABA_DATA_DIR` environment variable before launching the app. See [Configuration Files](./configuration-files.md) for details.

---

## Next steps

- [Getting Started](./getting-started.md) — Create and run your first task in three steps
- [Configuration Files](./configuration-files.md) — Understand `tasks.yaml`, `settings.yaml`, and the data directory
- [Troubleshooting](./troubleshooting.md) — Fixes for common installation problems
