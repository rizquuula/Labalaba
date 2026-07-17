# CLAUDE.md — Labalaba

## Overview
Labalaba is a cross-platform **process manager** with a Tauri desktop GUI. It spawns, monitors, and controls arbitrary OS processes (binaries, scripts, Python via runner prefixes) defined as "tasks". Tasks persist to `tasks.yaml` and support start/stop/restart, auto-restart on crash, cron scheduling, dependency ordering with startup delays, admin elevation (Windows UAC), live stdout/stderr log streaming, and per-task CPU/memory stats.

## Architecture
Cargo workspace (`resolver = "2"`) with three members + a SvelteKit frontend:

- `crates/shared` (`labalaba-shared`) — serde DTOs shared by daemon and GUI: `TaskConfig`, `TaskDto`, `TaskRequest`, `TaskStatus`, `TaskStats`, `AppSettings`, `LogEntry`/`LogStream`, `UpdateInfo`, `ApiResponse<T>`. `TaskStatus` serializes `snake_case`; `LogStream` is `lowercase`. **This is the contract** — the TS interfaces in `gui/src/lib/api/client.ts` mirror these by hand; keep them in sync.
- `crates/daemon` (`labalaba-daemon`) — the actual process-manager logic, structured **DDD / hexagonal** (`domain/`, `application/`, `infrastructure/`, `interface/`). Built as **both a lib and a standalone bin**.
- `gui/src-tauri` (`labalaba-gui`, lib name `tauri_app_lib`) — the Tauri 2 shell. Depends on `labalaba-daemon` as a library.
- `gui/` — SvelteKit 5 + TypeScript + Vite 6, static adapter (`@sveltejs/adapter-static`), xterm.js for the log viewer.

**Two runtime modes of the same daemon code:**
1. **Tauri app (the product):** `gui/src-tauri/src/lib.rs` calls `init_app_state(Some(log_cb), Some(update_cb))`, embedding the daemon **in-process**. The frontend talks to Rust via **Tauri commands** (`invoke`) and **Tauri events** (`listen`) — NOT HTTP. Log lines reach the UI through the `log_event_callback`, re-emitted as Tauri events named `log:{task_id}`; update availability emits `update-available`.
2. **Standalone daemon bin:** `crates/daemon/src/main.rs` runs an **axum HTTP + WebSocket server** on `127.0.0.1:{daemon_port}` (default `27015`). Routes in `interface/http/router.rs` (`/api/tasks`, `/api/stats`, `/api/settings`, `/api/update/check`, `/api/logs/{id}`) plus WS `/ws/logs/{id}`. This binary is the *alternate* transport; the GUI does not use it.

The daemon stays **Tauri-agnostic** — it knows nothing about Tauri; the bridge is the two optional `Fn` callbacks on `AppState` (`infrastructure/state.rs`).

**Key flow:** Tauri command (`commands/tasks.rs`) → application use case (one per file: `CreateTask`, `StartTask`, `StopTask`, `RestartTask`, `EditTask`, `DeleteTask`) → domain traits (`TaskRepository`, `ProcessSpawner`) → infrastructure impls (`YamlTaskRepository`, `OsProcessSpawner`, `GithubUpdater`, `LogFileWriter`, `ResourceMonitor`, `cron_scheduler`). `AppState` is `Arc`-shared; runtime status lives in `runtime_states` (in-memory `RwLock<HashMap>`, **not persisted**) while config persists to YAML.

## Key commands
The `Makefile` is the source of truth (it sets `LABALABA_DATA_DIR=$(CURDIR)` for dev). Note the Makefile's `stop`/`release-windows` targets are Windows-oriented.

```bash
make install        # cd gui && npm install
make dev            # Tauri app + hot-reload frontend (sets LABALABA_DATA_DIR to repo root)
make cargo-check    # cargo check -p labalaba-daemon && cargo check -p labalaba-shared
make check          # cd gui && npm run check  (svelte-kit sync + svelte-check, the TS type-check)
make test           # cargo test  (all Rust tests; CI scopes to -p labalaba-daemon -p labalaba-shared)
make build-be       # cargo build -p labalaba-daemon --release  (daemon bin only)
make clean          # cargo clean + rm gui/node_modules gui/.svelte-kit gui/build
```

GUI-only (run inside `gui/`): `npm run dev` (Vite on :1420), `npm run build`, `npm run check`, `npm run tauri <dev|build>`.

Full release bundle: `cd gui && npm run tauri build` (output in `gui/src-tauri/target/release/bundle/`).

## Conventions / things to know
- **Dev profile** (root `Cargo.toml`): `debug = "line-tables-only"`, `incremental = false` — deliberate, for faster Tauri dev builds. Don't "restore" these.
- **Version is synced across 5 files on release** — `gui/src-tauri/tauri.conf.json`, `gui/package.json`, and the three `Cargo.toml`s (gui, shared, daemon). The Release workflow rewrites them all from the git tag via `sed`. When bumping manually, update all five.
- **CI is mostly manual now.** `build.yml` and `test.yml` are `workflow_dispatch`-only (build is Linux-only; the win/mac matrix is commented out). `lint.yml` (`cargo audit` via rustsec) is the **only** workflow that runs on push/PR to `main` (path-filtered to `**/*.rs`, `**/Cargo.toml`, `Cargo.lock`) — and it's advisory (a failure doesn't block merge).
- **`release.yml` triggers on pushing a `v*` tag** — *not* on publishing a GitHub Release. `git push origin v1.2.5` builds and publishes a real, non-draft release. Its matrix is **5 targets**: win x64/x86, linux amd64/arm64, macOS **Silicon only** — there is no Intel Mac (`x86_64-apple-darwin`) build, and the run still reports success without one.
- **Data directory:** `LABALABA_DATA_DIR` env var sets where `tasks.yaml`, `settings.yaml`, and `logs/` live (defaults to CWD). `config_path`/`log_dir` in settings are resolved relative to it (`crates/daemon/src/lib.rs::resolve`).
- **Runner prefix** (`runner_prefix` on a task, e.g. `"uv run"` or `"python"`) wraps the executable: `exe=uv`, `args=["run", <executable>, ...original args]`. See `resolve_command` in `infrastructure/process/spawner.rs`.
- **Spawning is platform-split** (`spawner.rs`) — don't unify it. Unix runs children on a **PTY** (`spawn_on_pty`) so they line-buffer their logs; Windows spawns onto a **plain pipe** (`spawn_on_pipe`) with `CREATE_NO_WINDOW`, deliberately avoiding ConPTY, and sets `PYTHONUNBUFFERED=1` to keep Python flushing per line without a terminal. A packaged (Nuitka) Python binary was seen dying under ConPTY with `STATUS_DLL_INIT_FAILED` (`0xC0000142`) where a pipe spawn ran it fine — never reduced to a proven mechanism, so treat it as a caution. Either way `ProcessHandle.output` is *one* merged reader (on Windows stdout+stderr share a single `filedescriptor::Pipe`), which is why `start_task.rs` needs no `cfg`.
- **Crash recovery:** on startup `recover_task_states` re-checks persisted `pids` (`libc::kill(pid,0)` on Unix, `tasklist` on Windows) and marks tasks Running/Crashed. Auto-restart of crashed tasks goes through an `mpsc` channel (`restart_tx` → `restart_loop`) to avoid a recursive-`Send` issue.
- **`TaskRuntimeState` is not persisted** — it's rebuilt in memory each run; only `TaskConfig` (incl. `pids`) lives in YAML.
- **Tests live inline** (`#[cfg(test)] mod tests`) in shared and daemon; dev-deps `mockall` + `tempfile`. No frontend unit tests — `npm run check` (svelte-check) is the frontend gate.
- The README's "no HTTP, no sockets" claim describes the **shipped Tauri app**; the daemon crate still contains a full HTTP/WS server used only by the standalone bin.
