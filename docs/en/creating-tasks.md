# Creating Tasks

The complete reference for every field in the task form — from the basics to advanced scheduling.

---

## Opening the form

Click the **New Task** button at the top of the task list. If you have no tasks yet, click **Add your first task** instead. The **New Task** form opens as an overlay.

The form has two tabs: **Basic** and **Advanced**. Switch between them by clicking the tab labels or pressing the **Left** / **Right** arrow keys. Press **Escape** at any time to cancel without saving.

---

## Basic tab

### Description (required)

The display name shown on the task card throughout Labalaba.

- Placeholder: `My Application`
- Cannot be empty — the form will not submit without it.

### Executable / Script Path (required)

The path to the program or script you want to run.

- Placeholder: `Pick a binary, .py, .sh, .ps1, .bat…`
- Click **Browse** to open a file picker:
  - **Windows:** filters to `.exe`, `.bat`, `.cmd`, `.ps1`, `.py`, `.pyw`
  - **macOS / Linux:** shows all files (so extension-less binaries are selectable)
- When you pick a script (`.sh`, `.bat`, `.ps1`) via Browse, Labalaba **auto-detects its interpreter** and pre-fills the runner and arguments for you.

### Python Runner

This dropdown appears **only** when the path ends in `.py` or `.pyw`.

> **Note:** Hint shown: "Detected a Python script — it will be launched via this runner."

| Option | What it runs |
|---|---|
| `python` | The system `python` command |
| `pythonw` | Windows pythonw (no console window) |
| `uv run` | Run via the [uv](https://github.com/astral-sh/uv) package manager |
| `pipenv run python` | Run inside a Pipenv virtual environment |
| `poetry run python` | Run inside a Poetry virtual environment |
| `custom…` | Reveals a **Custom Runner Command** field — type any command (e.g., `uv run` or `/home/user/.venv/bin/python`) |

### Arguments

Space-separated command-line arguments passed to your program.

- Placeholder: `--port 8080 --config config.yaml`
- Arguments are split on spaces when the task is saved.

---

## Advanced tab

### Working Directory

The folder the program runs in. Leave blank to use the directory containing the executable.

- Placeholder: `C:\path\to\workdir`
- Click **Browse** to pick a folder.

### Environment Variables

Extra environment variables passed to the process. One `KEY=VALUE` pair per line.

```
NODE_ENV=production
PORT=8080
DATABASE_URL=postgres://localhost/myapp
```

> **Tip:** Values may contain `=` — only the **first** `=` on the line is treated as the separator. Lines without any `=` are ignored. Leading/trailing whitespace is trimmed.

### Cron Schedule

Run the task automatically on a schedule. Uses a 6-field cron expression.

- Placeholder: `0 */6 * * *` (optional)
- Example: `0 9 * * 1-5` — every weekday at 9:00 AM

See [Scheduling (Cron)](./scheduling.md) for the full format reference.

### Startup Delay (ms)

How many milliseconds Labalaba waits after receiving the start command before actually launching the process.

- Minimum: `0`
- Example: `5000` = 5 seconds

Useful when a task depends on another service that needs time to start. See [Dependencies & Startup Delay](./dependencies.md).

### Run as Admin

Check this box to launch the task with elevated (Administrator) privileges.

> **Warning:** Only enable this for programs that genuinely require elevated access. See [Admin Elevation](./admin-elevation.md).

### Auto-restart on crash

Check this box to have Labalaba automatically restart the task if it exits with a non-zero error code.

> **Note:** A clean exit (exit code 0) is **not** treated as a crash — auto-restart will not trigger. See [Auto-restart](./auto-restart.md).

---

## Submitting the form

| Button | Shown when | Loading label |
|---|---|---|
| **Create Task** | Creating a new task | "Creating…" |
| **Save Changes** | Editing an existing task | "Saving…" |
| **Cancel** | Always | — |

If validation fails (e.g., missing **Description** or **Executable / Script Path**), or if a save error occurs, a red error message appears above the footer buttons.

---

## How the runner prefix works

When you set a **Python Runner** (or Labalaba auto-fills one for a script), it becomes a "runner prefix". The first word of the runner is the command; the rest become leading arguments. Your executable and your own arguments follow.

| Runner | Executable | Arguments | Actually runs |
|---|---|---|---|
| `uv run` | `script.py` | `--port 8080` | `uv run script.py --port 8080` |
| `python` | `app.py` | `--verbose` | `python app.py --verbose` |
| `pipenv run python` | `main.py` | (none) | `pipenv run python main.py` |
| `node` | `server.js` | `--inspect` | `node server.js --inspect` |
| (none) | `bash` | `run.sh` | `bash run.sh` |

The **Python Runner** dropdown is a convenient shortcut for `.py` files. For other script types, the **Browse** auto-fill handles it. You can also type a runner directly into the **Custom Runner Command** field.

---

## Field reference

### Basic tab

| Field | Required | Type | Notes |
|---|---|---|---|
| **Description** | Yes | Text | Display name; cannot be empty |
| **Executable / Script Path** | Yes | File path | Browse available; auto-detects runner for scripts |
| **Python Runner** | — | Dropdown | Appears for `.py`/`.pyw` files only |
| **Custom Runner Command** | — | Text | Appears when **Python Runner** = `custom…` |
| **Arguments** | No | Text | Space-separated; split on save |

### Advanced tab

| Field | Required | Type | Notes |
|---|---|---|---|
| **Working Directory** | No | Folder path | Browse available; defaults to executable's directory |
| **Environment Variables** | No | Textarea | One `KEY=VALUE` per line |
| **Cron Schedule** | No | Text | 6-field cron expression; see [Scheduling](./scheduling.md) |
| **Startup Delay (ms)** | No | Number (≥ 0) | Milliseconds to wait before launch |
| **Run as Admin** | No | Checkbox | Elevates privileges; see [Admin Elevation](./admin-elevation.md) |
| **Auto-restart on crash** | No | Checkbox | Restarts on non-zero exit; see [Auto-restart](./auto-restart.md) |

---

## Worked examples

### Example 1 — Node.js web server

| Field | Value |
|---|---|
| **Description** | `API Server` |
| **Executable / Script Path** | `/home/user/myapp/server.js` |
| **Arguments** | `--port 3000` |
| **Working Directory** | `/home/user/myapp` |
| **Environment Variables** | `NODE_ENV=production` |

In the **Arguments** field, set the runner by typing `node` in the **Custom Runner Command** (choose `custom…` from the **Python Runner** dropdown… but wait — the Python Runner only appears for `.py` files). For a `.js` file, place `node` as the **Executable / Script Path** and `server.js --port 3000` as **Arguments**, or keep the path as `server.js` and set a custom runner of `node`.

> **Tip:** The simplest approach for Node scripts is: **Executable / Script Path** = the full path to `node` (e.g., `/usr/bin/node`) and **Arguments** = `server.js --port 3000`, with **Working Directory** set to your project folder.

### Example 2 — Python app via `uv run`

| Field | Value |
|---|---|
| **Description** | `Data Pipeline` |
| **Executable / Script Path** | `/home/user/pipeline/main.py` |
| **Python Runner** | `uv run` |
| **Arguments** | `--env production` |
| **Working Directory** | `/home/user/pipeline` |
| **Auto-restart on crash** | Checked |

Labalaba will run: `uv run /home/user/pipeline/main.py --env production`

### Example 3 — Shell script on Linux

| Field | Value |
|---|---|
| **Description** | `Backup Script` |
| **Executable / Script Path** | `/home/user/scripts/backup.sh` (picked via **Browse**) |
| **Cron Schedule** | `0 2 * * *` (every day at 2:00 AM) |

Browsing to `backup.sh` auto-detects `bash` as the runner and pre-fills it for you.

---

## Related

- [Getting Started](./getting-started.md) — Quick walkthrough of your first task
- [Managing Tasks](./managing-tasks.md) — Start, stop, edit, delete, and search
- [Auto-restart](./auto-restart.md) — Keep tasks alive after a crash
- [Scheduling (Cron)](./scheduling.md) — Run tasks on a timed schedule
- [Dependencies & Startup Delay](./dependencies.md) — Control start order and timing
- [Admin Elevation](./admin-elevation.md) — Running tasks with elevated privileges
