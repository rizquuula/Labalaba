# Labalaba - Cross-Platform Process Manager

## Context

Build a desktop application for spawning, managing, and monitoring processes (initially Windows .exe, designed for cross-platform). The user needs a modern GUI to configure tasks, start/stop/restart them, view real-time logs, and persist configuration across restarts. The app itself should support hot-reload updates without interrupting running tasks.

## Decisions Made

- **Stack**: Tauri (Rust) + Svelte + TypeScript
- **Architecture**: GUI + Daemon separation (tasks survive GUI restarts/updates)
- **IPC**: Local HTTP (axum) + WebSocket for log streaming
- **Daemon design**: Domain-Driven Design (DDD) with 4 layers
- **GUI design**: Simple thin-client (components/stores/api)
- **Theme**: Glassmorphism with light/dark toggle (default dark)
- **Config**: YAML file in CWD, tasks persist but require manual start
- **File constraint**: Max 200-300 lines per file, split into submodules when exceeded

---

## Architecture Overview

```
┌─────────────────────────────┐
│     Labalaba GUI (Tauri)    │
│  ┌───────────────────────┐  │
│  │   Svelte + TypeScript │  │
│  │   Glassmorphism Theme │  │
│  └───────────┬───────────┘  │
│  ┌───────────┴───────────┐  │
│  │  Tauri Rust Backend   │  │
│  │  (thin proxy layer)   │  │
│  └───────────┬───────────┘  │
└──────────────┼──────────────┘
               │ HTTP + WebSocket (localhost)
┌──────────────┼──────────────┐
│     Labalaba Daemon         │
│  ┌─────────────────────┐    │
│  │ Interface (axum)    │    │
│  │ Application (usecases)│   │
│  │ Domain (entities)   │    │
│  │ Infrastructure      │    │
│  └─────────────────────┘    │
└─────────────────────────────┘
         │
   ┌─────┼─────┐
 Task1 Task2  TaskN
```

---

## Project Structure

```
labalaba/
├── Cargo.toml                       # Workspace: daemon + shared + gui/src-tauri
├── crates/
│   ├── daemon/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs              # Bootstrap, DI, start HTTP server
│   │       ├── domain/
│   │       │   ├── mod.rs
│   │       │   ├── task/
│   │       │   │   ├── mod.rs
│   │       │   │   ├── entity.rs    # Task entity, TaskId value object
│   │       │   │   ├── status.rs    # TaskStatus enum, state transitions
│   │       │   │   └── repository.rs# TaskRepository trait (port)
│   │       │   ├── process/
│   │       │   │   ├── mod.rs
│   │       │   │   ├── entity.rs    # ProcessHandle, ProcessInfo
│   │       │   │   └── service.rs   # ProcessSpawner trait (port)
│   │       │   ├── scheduler/
│   │       │   │   ├── mod.rs
│   │       │   │   ├── schedule.rs  # Schedule value object (cron)
│   │       │   │   └── service.rs   # SchedulerService trait (port)
│   │       │   └── log/
│   │       │       ├── mod.rs
│   │       │       └── entity.rs    # LogEntry, LogStream trait (port)
│   │       ├── application/
│   │       │   ├── mod.rs
│   │       │   ├── task/
│   │       │   │   ├── mod.rs
│   │       │   │   ├── create_task.rs
│   │       │   │   ├── start_task.rs
│   │       │   │   ├── stop_task.rs
│   │       │   │   ├── restart_task.rs
│   │       │   │   ├── edit_task.rs
│   │       │   │   └── delete_task.rs
│   │       │   ├── log/
│   │       │   │   ├── mod.rs
│   │       │   │   └── stream_logs.rs
│   │       │   ├── update/
│   │       │   │   ├── mod.rs
│   │       │   │   └── check_update.rs
│   │       │   └── dto.rs           # Data Transfer Objects
│   │       ├── infrastructure/
│   │       │   ├── mod.rs
│   │       │   ├── persistence/
│   │       │   │   ├── mod.rs
│   │       │   │   └── yaml_repository.rs
│   │       │   ├── process/
│   │       │   │   ├── mod.rs
│   │       │   │   ├── spawner.rs
│   │       │   │   └── admin.rs     # runas elevation (Windows)
│   │       │   ├── scheduler/
│   │       │   │   ├── mod.rs
│   │       │   │   └── cron_scheduler.rs
│   │       │   ├── log/
│   │       │   │   ├── mod.rs
│   │       │   │   └── collector.rs
│   │       │   └── updater/
│   │       │       ├── mod.rs
│   │       │       └── github_updater.rs
│   │       └── interface/
│   │           ├── mod.rs
│   │           ├── http/
│   │           │   ├── mod.rs
│   │           │   ├── router.rs
│   │           │   ├── task_handler.rs
│   │           │   ├── settings_handler.rs
│   │           │   └── update_handler.rs
│   │           └── ws/
│   │               ├── mod.rs
│   │               └── log_handler.rs
│   └── shared/                      # Shared types between daemon & GUI
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── task.rs              # Task DTOs, TaskStatus enum
│           └── api.rs               # API request/response types
├── gui/
│   ├── src-tauri/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       └── commands.rs          # Tauri commands (proxy to daemon)
│   ├── src/
│   │   ├── App.svelte
│   │   ├── app.css                  # Global styles
│   │   ├── lib/
│   │   │   ├── components/
│   │   │   │   ├── TopBar.svelte        # Stats + theme toggle + settings
│   │   │   │   ├── TaskList.svelte      # Task card grid
│   │   │   │   ├── TaskCard.svelte      # Individual task card
│   │   │   │   ├── TaskForm.svelte      # Add/Edit task dialog
│   │   │   │   ├── LogViewer.svelte     # Terminal-style log panel
│   │   │   │   ├── Settings.svelte      # Settings page
│   │   │   │   └── StatsBar.svelte      # Running/Stopped/Total
│   │   │   ├── stores/
│   │   │   │   ├── tasks.ts             # Task state store
│   │   │   │   ├── theme.ts             # Light/dark theme store
│   │   │   │   ├── logs.ts              # Log buffer store
│   │   │   │   └── settings.ts          # Settings store
│   │   │   └── api/
│   │   │       ├── client.ts            # HTTP client to daemon
│   │   │       └── websocket.ts         # WebSocket log connection
│   │   └── styles/
│   │       ├── theme.css                # CSS custom properties (light/dark)
│   │       └── glassmorphism.css        # Glass effect utilities
│   ├── package.json
│   ├── tsconfig.json
│   ├── vite.config.ts
│   └── svelte.config.js
└── tasks.yaml                       # Runtime task persistence (CWD)
```

---

## Task YAML Schema

```yaml
tasks:
  - id: "550e8400-e29b-41d4-a716-446655440000"
    name: "My App"
    executable: "C:\\path\\to\\app.exe"
    arguments: ["--port", "8080"]
    working_directory: "C:\\path\\to"
    environment:
      NODE_ENV: "production"
    run_as_admin: false
    auto_restart: true
    schedule: null            # or cron: "0 */5 * * *"
    startup_delay_ms: 0
    depends_on: []            # list of task IDs to start first
```

---

## Daemon API Design

### REST Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/tasks` | List all tasks with status |
| POST | `/api/tasks` | Create new task |
| GET | `/api/tasks/:id` | Get task detail |
| PUT | `/api/tasks/:id` | Update task config |
| DELETE | `/api/tasks/:id` | Delete task |
| POST | `/api/tasks/:id/start` | Start task |
| POST | `/api/tasks/:id/stop` | Stop task |
| POST | `/api/tasks/:id/restart` | Restart task |
| GET | `/api/stats` | Get summary stats (running/stopped/total) |
| GET | `/api/settings` | Get app settings |
| PUT | `/api/settings` | Update app settings |
| POST | `/api/update/check` | Check for updates |
| POST | `/api/update/apply` | Apply available update |

### WebSocket

| Path | Description |
|------|-------------|
| `ws://localhost:{port}/ws/logs/:task_id` | Stream real-time stdout/stderr for a task |

---

## Key Features Detail

### Admin Elevation
- Windows: Use `runas` crate to spawn elevated processes
- Linux/macOS: `pkexec` or `sudo` (future)
- Per-task toggle in config

### Hot-Reload Update Flow
1. GUI Settings -> "Check for Updates" -> calls `POST /api/update/check`
2. Daemon checks GitHub releases API for newer version
3. If available, GUI shows update prompt -> calls `POST /api/update/apply`
4. Daemon downloads new GUI binary to temp location
5. Daemon replaces GUI binary, signals GUI to restart
6. GUI restarts with new version, reconnects to daemon
7. All tasks continue running uninterrupted

### Log Viewer
- Terminal-style with ANSI color support via `xterm.js` or similar
- Auto-scroll with ability to pause/scroll back
- Per-task log buffer (configurable max lines in settings)
- Streams via WebSocket from daemon

### Task Dependencies & Scheduling
- `depends_on`: When starting a task, start its dependencies first (with delay)
- `schedule`: Cron-like expression for periodic execution
- `startup_delay_ms`: Wait before spawning after start command

### Desktop Notifications
- Notify on task crash / unexpected stop
- Notify on update available
- Configurable in settings (on/off)

---

## Settings Schema

```yaml
settings:
  theme: "dark"                    # "dark" | "light"
  daemon_port: 27015               # localhost port for HTTP+WS
  log_buffer_lines: 5000           # max log lines per task
  config_path: "./tasks.yaml"      # path to task config
  notifications_enabled: true
  auto_check_updates: true
  update_check_interval_hours: 24
  launch_on_startup: false
```

---

## Rust Crate Dependencies (Key)

### Daemon
- `axum` - HTTP framework
- `tokio` - Async runtime
- `tokio-tungstenite` - WebSocket
- `serde` / `serde_yaml` - YAML serialization
- `uuid` - Task IDs
- `chrono` - Timestamps
- `cron` - Cron expression parsing
- `runas` - Windows admin elevation
- `reqwest` - HTTP client (update checker)
- `tracing` - Logging

### GUI (src-tauri)
- `tauri` - Desktop framework
- `reqwest` - HTTP client to daemon
- `shared` (workspace crate) - Shared types

### Frontend (npm)
- `svelte` + `@sveltejs/vite-plugin-svelte`
- `typescript`
- `xterm` - Terminal log viewer
- `@tauri-apps/api` - Tauri JS bindings

---

## Verification Plan

1. **Daemon standalone**: `cargo run -p labalaba-daemon` -> verify HTTP API responds on localhost
2. **GUI launch**: `cargo tauri dev` -> verify GUI opens, connects to daemon
3. **Task CRUD**: Create/edit/delete a task via GUI, verify `tasks.yaml` updates
4. **Process spawn**: Add a simple .exe (e.g., `ping localhost`), start it, verify logs stream
5. **Admin elevation**: Toggle run-as-admin, verify UAC prompt appears on Windows
6. **Persistence**: Restart app, verify tasks appear but are stopped
7. **Theme toggle**: Switch light/dark, verify glassmorphism renders in both
8. **Update check**: Mock a GitHub release, verify update flow works
