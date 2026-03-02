# Labalaba Launcher Design

**Date:** 2026-03-02
**Status:** Approved

---

## Overview

A standalone `labalaba` Rust binary that orchestrates the daemon and GUI as child processes. When killed (Ctrl+C or SIGTERM), it gracefully stops all managed tasks via the daemon HTTP API, requests daemon shutdown via a new `/api/shutdown` endpoint, waits for clean exit, then kills the GUI.

---

## Architecture

```
labalaba (launcher)
    │
    ├── spawns → labalaba-daemon        (child process)
    │               └── HTTP on localhost:{port}
    │
    └── spawns → labalaba-gui           (child process, Tauri)

Shutdown sequence (on Ctrl+C / SIGTERM):
    1. GET  /api/tasks                  → collect running task IDs
    2. POST /api/tasks/:id/stop         → stop each concurrently (3s timeout per task)
    3. POST /api/shutdown               → new daemon endpoint, triggers graceful exit
    4. Wait for daemon process exit     → 5s timeout, then SIGKILL
    5. Kill GUI process                 → immediate terminate
```

---

## New Crate: `crates/launcher`

Added to the workspace as a new member producing a `labalaba` binary.

### File Structure

```
crates/launcher/
├── Cargo.toml
└── src/
    ├── main.rs       # Entry point: parse args, spawn children, wait, drive shutdown
    ├── config.rs     # Read settings.yaml to get port and binary paths
    ├── health.rs     # Poll /api/stats until daemon is ready (with timeout)
    └── shutdown.rs   # Graceful shutdown: stop tasks → /api/shutdown → wait/kill
```

### CLI Interface

```
labalaba [OPTIONS]

Options:
  --daemon   <path>   Path to labalaba-daemon binary  [default: ./labalaba-daemon(.exe)]
  --gui      <path>   Path to labalaba-gui binary     [default: ./labalaba-gui(.exe)]
  --settings <path>   Path to settings.yaml           [default: ./settings.yaml]
```

### Dependencies (all workspace or minimal additions)

```toml
tokio          = { workspace = true }
reqwest        = { workspace = true }
serde_yaml     = { workspace = true }
serde          = { workspace = true }
anyhow         = { workspace = true }
tracing        = { workspace = true }
tracing-subscriber = { workspace = true }
labalaba-shared = { path = "../shared" }
```

---

## Daemon Changes: `/api/shutdown` Endpoint

A new `POST /api/shutdown` endpoint is added to the daemon. It signals the axum server to stop via a `tokio::sync::oneshot::Sender<()>` stored in `AppState`. The server is started with `.with_graceful_shutdown(...)` listening on that channel.

### Changes Required

- `infrastructure/state.rs` — add `shutdown_tx: Mutex<Option<oneshot::Sender<()>>>` to `AppState`
- `interface/http/router.rs` — register `POST /api/shutdown` handler
- `interface/http/shutdown_handler.rs` — new handler: fires the oneshot sender
- `main.rs` — create oneshot pair, store sender in state, pass receiver to `with_graceful_shutdown`

---

## Signal Handling (Cross-Platform)

```rust
// All platforms
tokio::signal::ctrl_c()

// Unix only
#[cfg(unix)]
tokio::signal::unix::signal(SignalKind::terminate())
```

The launcher selects whichever fires first using `tokio::select!`.

---

## Health Check

After spawning the daemon, the launcher polls `GET /api/stats` every 200ms for up to 10 seconds. If the daemon does not respond within that window, the launcher aborts with an error.

---

## Shutdown Timing & Timeouts

| Step                        | Timeout  | Fallback           |
|-----------------------------|----------|--------------------|
| Stop each task (concurrent) | 3s total | log warning, continue |
| Daemon graceful exit        | 5s       | SIGKILL daemon     |
| GUI exit                    | —        | immediate kill     |

---

## Binary Defaults (Platform-Aware)

```rust
#[cfg(windows)]
const DAEMON_BIN: &str = "labalaba-daemon.exe";
#[cfg(not(windows))]
const DAEMON_BIN: &str = "labalaba-daemon";
```

Binaries are resolved relative to the launcher's own executable directory, falling back to the current working directory.

---

## Verification Plan

1. `cargo build -p labalaba-launcher` — builds cleanly
2. `./labalaba` — daemon starts, GUI opens, launcher logs both PIDs
3. `Ctrl+C` — tasks stop, daemon exits cleanly, GUI closes
4. `kill -TERM <launcher_pid>` (Unix) — same clean shutdown
5. Daemon unresponsive scenario — launcher falls back to SIGKILL after 5s
6. `--daemon` / `--gui` flags — custom paths respected
