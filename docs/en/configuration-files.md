# Configuration Files

Labalaba stores everything it needs — your tasks, your settings, and your log files — in a single folder called the **data directory**.

---

## The data directory

By default the data directory is the app's working directory (the same folder as the Labalaba binary in a normal installation). Everything lives there:

| Item | Default path | What it contains |
|---|---|---|
| `tasks.yaml` | `./tasks.yaml` | All your task definitions |
| `settings.yaml` | `./settings.yaml` | All app settings |
| `logs/` | `./logs/` | Per-task log files |

### Changing the data directory

Set the `LABALABA_DATA_DIR` environment variable before launching Labalaba to point the app at a different folder:

```
LABALABA_DATA_DIR=/home/you/labalaba-data
```

Any relative paths in `settings.yaml` (like `./tasks.yaml` or `./logs`) resolve against this directory.

> **Tip:** This is useful if you want to keep your data on a shared drive or a specific user folder.

---

## tasks.yaml

This file stores every task you have created. It is read when Labalaba starts and updated whenever you create, edit, or delete a task in the UI.

### Example

```yaml
tasks:
  - id: "550e8400-e29b-41d4-a716-446655440000"
    description: "My API Server"
    executable: "C:\\Apps\\server.exe"
    arguments: ["--port", "8080"]
    working_directory: "C:\\Apps"
    environment:
      NODE_ENV: "production"
    run_as_admin: false
    auto_restart: true
    schedule: null
    startup_delay_ms: 0
    depends_on: []
    runner_prefix: null
    pids: []
```

### Field reference

| Field | What it stores | Notes |
|---|---|---|
| `id` | Unique identifier for the task | Auto-generated — do not change this value |
| `description` | The display name shown in the UI | |
| `executable` | Full path to the program or script | |
| `arguments` | List of command-line arguments | |
| `working_directory` | Folder the task runs in | Omit to use the app's working directory |
| `environment` | Key/value map of environment variables | |
| `run_as_admin` | `true` to elevate (Windows UAC) | `false` on macOS/Linux (no effect) |
| `auto_restart` | `true` to restart on unexpected exit | |
| `schedule` | Cron string (6-field) or `null` | See [Scheduling](./scheduling.md) |
| `startup_delay_ms` | Milliseconds to wait before starting | Useful with `depends_on` |
| `depends_on` | List of task `id` values that must start first | See [Dependencies](./dependencies.md) |
| `runner_prefix` | Interpreter prefix, e.g. `"uv run"` | `null` to run directly |
| `pids` | Process IDs of the running task | Managed automatically — leave as `[]` |

> **Note:** The `depends_on` field can only be set by hand-editing `tasks.yaml` — there is no UI for it yet. See [Dependencies & Startup Delay](./dependencies.md) for details.

---

## settings.yaml

This file stores all your app preferences. It is updated whenever you click **Save Settings** in the UI.

### Example (showing defaults)

```yaml
theme: "dark"
daemon_port: 27015
log_buffer_lines: 5000
config_path: "./tasks.yaml"
notifications_enabled: true
auto_check_updates: true
update_check_interval_hours: 24
launch_on_startup: false
log_dir: "./logs"
log_max_file_size_mb: 10
log_max_rotated_files: 5
```

### Field reference

| Field | Default | What it controls |
|---|---|---|
| `theme` | `"dark"` | `"dark"` or `"light"` |
| `daemon_port` | `27015` | Internal engine port (1024–65535) |
| `log_buffer_lines` | `5000` | Max log lines in memory per task (100–50000) |
| `config_path` | `"./tasks.yaml"` | Path to the task definitions file |
| `notifications_enabled` | `true` | Desktop crash/stop alerts on or off |
| `auto_check_updates` | `true` | Check for updates once a day |
| `update_check_interval_hours` | `24` | Hours between automatic update checks |
| `launch_on_startup` | `false` | Start Labalaba when you log in |
| `log_dir` | `"./logs"` | Folder for per-task log files |
| `log_max_file_size_mb` | `10` | Rotate a log file after it reaches this size (MB) |
| `log_max_rotated_files` | `5` | Old log files to keep per task (0 = none) |

---

## Hand-editing the files

Both files are plain YAML and can be opened in any text editor. This is the only way to set some advanced options (such as `depends_on` in `tasks.yaml`).

> **Warning:** Always close Labalaba before editing `tasks.yaml` or `settings.yaml`. If the app is running it may overwrite your changes when it next saves.

> **Tip:** Before making significant edits, copy both files to a safe location as a backup. To move your entire setup to another computer, copy `tasks.yaml` and `settings.yaml` to the same relative location on the new machine.

---

## Related

- [Settings](./settings.md)
- [Dependencies & Startup Delay](./dependencies.md)
- [Scheduling (Cron)](./scheduling.md)
- [Troubleshooting](./troubleshooting.md)
- [Back to home](./README.md)
