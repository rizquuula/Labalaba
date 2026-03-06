<div align="center">

<img src="assets/labalaba-logo.jpg" alt="Labalaba Logo" width="96" height="96" />

# Labalaba

**A modern, cross-platform process manager with a glassmorphism UI**

[![Build](https://img.shields.io/github/actions/workflow/status/rizquuula/labalaba/ci.yml?style=flat-square&logo=github)](https://github.com/rizquuula/labalaba/actions)
[![Release](https://img.shields.io/github/v/release/rizquuula/labalaba?style=flat-square&color=blue)](https://github.com/rizquuula/labalaba/releases)
[![License](https://img.shields.io/github/license/rizquuula/labalaba?style=flat-square)](LICENSE)
[![Stars](https://img.shields.io/github/stars/rizquuula/labalaba?style=flat-square&color=yellow)](https://github.com/rizquuula/labalaba/stargazers)
[![Issues](https://img.shields.io/github/issues/rizquuula/labalaba?style=flat-square)](https://github.com/rizquuula/labalaba/issues)
[![Made with Rust](https://img.shields.io/badge/made%20with-Rust-orange?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![Tauri](https://img.shields.io/badge/powered%20by-Tauri-24C8DB?style=flat-square&logo=tauri)](https://tauri.app/)

<br/>

> Spawn, monitor, and manage your apps — all from one beautiful desktop GUI.
> Labalaba keeps your tasks running in the background while you stay in control.

<br/>

[**Download**](https://github.com/rizquuula/labalaba/releases) · [Report Bug](https://github.com/rizquuula/labalaba/issues) · [Request Feature](https://github.com/rizquuula/labalaba/issues)

<br/>

<!-- Replace with actual screenshot once available -->
<!-- <img src="docs/assets/screenshot-dark.png" alt="Labalaba Screenshot" width="780" /> -->

</div>

---

## ✨ Features

| Feature | Description |
|---|---|
| 🚀 **Process Spawner** | Launch any `.exe`, script, or binary with custom args and environment |
| 📋 **Task Persistence** | Tasks saved to `tasks.yaml` — survive restarts, start manually |
| 📡 **Real-time Logs** | Live terminal-style log viewer with stdout/stderr per task |
| 🔄 **Start / Stop / Restart** | Full lifecycle control with one click |
| 🛡️ **Admin Elevation** | Run tasks as Administrator via UAC (Windows) |
| 🔁 **Auto-restart** | Automatically restart crashed processes |
| ⏰ **Cron Scheduling** | Schedule tasks with cron expressions |
| 🔗 **Task Dependencies** | Start tasks in order with configurable delays |
| 🌓 **Light / Dark Theme** | Glassmorphism UI with smooth theme toggle |
| 🔔 **Notifications** | Desktop alerts on crash or unexpected stop |
| 📦 **Single Binary** | One installer — no separate daemon process to manage |
| 📊 **Stats Bar** | Live counts of running / stopped / crashed tasks |

---

## 🏗️ Architecture

Labalaba runs as a **single Tauri process** — the daemon logic is embedded directly inside the app. The frontend communicates with the Rust backend via Tauri commands and events (no HTTP, no sockets).

```
┌─────────────────────────────────────────┐
│           Labalaba (Tauri App)           │
│                                          │
│  ┌───────────────────────────────────┐  │
│  │  SvelteKit UI (WebView)           │  │  ← Svelte + TypeScript
│  │  invoke() · listen()              │  │
│  └──────────────┬────────────────────┘  │
│                 │ Tauri commands/events  │
│  ┌──────────────┴────────────────────┐  │
│  │  Daemon Logic (Rust / tokio)      │  │  ← DDD architecture
│  │  AppState · Use Cases             │  │    Embedded in Tauri process
│  │  YAML persistence · Log streaming │  │
│  └──────────────┬────────────────────┘  │
└─────────────────┼────────────────────────┘
                  │ std::process::Command
         ┌────────┼────────┐
       Task1    Task2    TaskN  ← OS processes, managed by PID
```

The daemon logic is built with **Domain-Driven Design** (DDD):

```
crates/daemon/src/
├── domain/          # Entities, value objects, repository traits
├── application/     # One use case per file (StartTask, StopTask, …)
├── infrastructure/  # YAML persistence, process spawner, log collector
└── interface/       # axum HTTP handlers (used by standalone daemon only)
```

---

## 📦 Installation

### Download Binary *(recommended)*

Grab the latest release for your platform:

| Platform | Download |
|---|---|
| Windows (x64) | [labalaba-windows-x64.msi](https://github.com/rizquuula/labalaba/releases) |
| Linux (x64) | [labalaba-linux-x64.AppImage](https://github.com/rizquuula/labalaba/releases) |
| macOS | [labalaba-macos.dmg](https://github.com/rizquuula/labalaba/releases) |

### Build from Source

**Prerequisites:** Rust 1.75+, Node.js 18+, npm

```bash
git clone https://github.com/rizquuula/labalaba.git
cd labalaba

make install   # install frontend npm dependencies
make dev       # dev mode: Tauri app + hot-reload frontend
make build     # release build (produces installer in gui/src-tauri/target/release/bundle/)
```

---

## 🚀 Quick Start

**1. Launch the app**

Open Labalaba. The daemon starts automatically inside the app — nothing else to run.

**2. Add a task**

Click **New Task** → fill in the executable path → **Create Task**.

**3. Start it**

Hit ▶ **Start** on the task card. Logs stream in real time.

---

## ⚙️ Configuration

Tasks are stored in `tasks.yaml` in the working directory (repo root in dev, next to the binary in production):

```yaml
tasks:
  - id: "550e8400-e29b-41d4-a716-446655440000"
    name: "My API Server"
    executable: "C:\\Apps\\server.exe"
    arguments: ["--port", "8080"]
    working_directory: "C:\\Apps"
    environment:
      NODE_ENV: "production"
    run_as_admin: false
    auto_restart: true
    schedule: null           # or cron: "0 */6 * * *"
    startup_delay_ms: 0
    depends_on: []
```

App settings are in `settings.yaml`:

```yaml
theme: "dark"              # "dark" | "light"
log_buffer_lines: 5000
notifications_enabled: true
auto_check_updates: true
```

> **Data directory:** set `LABALABA_DATA_DIR` to override where `tasks.yaml`, `settings.yaml`, and `logs/` are stored.

---

## 🛠️ Tech Stack

| Layer | Technology |
|---|---|
| **GUI** | [Tauri 2](https://tauri.app/) + [SvelteKit 5](https://svelte.dev/) + TypeScript |
| **Backend** | Rust + [tokio](https://tokio.rs/) — embedded in the Tauri process |
| **IPC** | Tauri commands (`invoke`) + Tauri events (`listen`) |
| **Persistence** | YAML (`serde_yaml`) |
| **Scheduling** | Cron expressions (`cron` crate) |
| **Styling** | Glassmorphism CSS with CSS custom properties |

---

## 📈 Star History

<a href="https://star-history.com/#rizquuula/labalaba&Date">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=rizquuula/labalaba&type=Date&theme=dark" />
    <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=rizquuula/labalaba&type=Date" />
    <img alt="Star History Chart" src="https://api.star-history.com/svg?repos=rizquuula/labalaba&type=Date" />
  </picture>
</a>

---

## 🤝 Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) first.

```bash
# Fork the repo, then:
git clone https://github.com/YOUR_USERNAME/labalaba.git
cd labalaba

# Create a feature branch
git checkout -b feat/my-feature

# Make changes, then run checks
cargo check -p labalaba-daemon
cd gui && npm run check

# Commit and open a PR
git commit -m "feat: add my feature"
git push origin feat/my-feature
```

### Project Structure

```
labalaba/
├── crates/
│   ├── daemon/        # Process manager logic (Rust, DDD) — lib + standalone bin
│   └── shared/        # Shared types (DTOs, API models)
├── gui/
│   ├── src/           # SvelteKit frontend
│   │   ├── lib/
│   │   │   ├── api/         # Tauri invoke/listen clients
│   │   │   ├── components/  # UI components
│   │   │   └── stores/      # Svelte stores (tasks, theme, settings)
│   │   └── styles/    # Glassmorphism + theme CSS
│   └── src-tauri/     # Tauri app — embeds daemon logic + Tauri commands
└── docs/              # Design documents & assets
```

---

## 📄 License

MIT © [M Razif Rizqullah](https://github.com/rizquuula)

---

<div align="center">

**If Labalaba saves you time, consider giving it a ⭐**

Made with ❤️ and a lot of ☕ in Rust

</div>
