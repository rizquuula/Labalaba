# Managing Tasks

Start, stop, restart, search, and monitor your tasks — everything the task list and task cards can do.

---

## The stats bar

The top bar always shows a live summary of all your tasks:

| Indicator | Colour | Meaning |
|---|---|---|
| **Running** | Green | Tasks currently active |
| **Stopped** | Gray | Tasks not running |
| **Crashed** | Red | Tasks that exited with an error |
| **Total** | — | Total number of tasks defined |

The counts update automatically about every 2 seconds.

---

## The task list

Below the **TASKS** header you'll find every task you've created. The list auto-refreshes approximately every 2 seconds, so status changes appear quickly without any manual reload.

### Searching and filtering

Once you have at least one task, two controls appear above the list:

- **Search tasks…** — type any part of a task name to narrow the list instantly.
- **Status filter dropdown** — pick **All statuses**, **Running**, **Stopped**, or **Crashed** to show only tasks in that state.

If no tasks match your search, the list shows: "No tasks match your search."

---

## Task card anatomy

Each task is shown as a card. Here is what each part means:

```
┌─────────────────────────────────────────────────────────┐
│  [STATUS BADGE]  Task Name                              │
│  /path/to/executable   PID 12345   CPU: 2.1%   Mem: 45MB│
│  [ADMIN]  [AUTO-RESTART]                                │
│  [ Stop ] [ Restart ] [ View Logs ] [ Edit ] [ Delete ] │
└─────────────────────────────────────────────────────────┘
```

| Element | Details |
|---|---|
| **Status badge** | Color-coded pill showing current state (see table below) |
| Task name | The **Description** you gave the task |
| Executable path | Shown in monospace; visible when available |
| **PID** | Process ID of the running process (running tasks only) |
| **CPU** | Current CPU usage as a percentage (running tasks only) |
| **Memory** | Resident memory usage in MB (running tasks only) |
| **ADMIN** tag | Shown if the task is configured to run as Administrator |
| **AUTO-RESTART** tag | Shown if auto-restart on crash is enabled |
| Card border | Tints **green** when running, **red** when crashed |

CPU and memory figures refresh about every 5 seconds while the task is running.

---

## Status badge meanings

| Status | Colour | What it means |
|---|---|---|
| **stopped** | Gray | The task is not running |
| **starting** | Yellow | Launch is in progress (including any configured startup delay) |
| **running** | Green | The task is alive and being monitored |
| **stopping** | — | A stop was requested; the process is shutting down |
| **crashed** | Red | The task exited with a non-zero (error) code and was not auto-restarted |

> **Note:** A task that exits with **code 0** is treated as a normal, intentional stop — not a crash. It will show as **stopped** (gray), even if auto-restart is enabled.

---

## Starting a task

1. Find the task card (use the search box or scroll).
2. Click **Start**.

The badge moves from **stopped** → **starting** → **running** as the process launches.

---

## Stopping a task

1. Click **Stop** on a running task card.
2. A confirmation dialog titled **Stop Task** appears.
3. Confirm to stop the process.

The badge moves to **stopping**, then **stopped** once the process has exited.

---

## Restarting a task

1. Click **Restart** on a running task card.
2. A confirmation dialog appears: "stop and start it again".
3. Confirm to restart.

Labalaba stops the process and immediately starts it again.

> **Tip:** Use **Restart** after changing a config file that the process reads on startup — no need to stop and start manually.

---

## Viewing logs

Click **View Logs** on any task card to open the live log viewer. Stdout and stderr from the process stream in real time.

Press **Escape** to close the log viewer. See [Viewing Logs](./logs.md) for more, including how to access log files on disk.

---

## Editing a task

1. Click **Edit** on any task card.
2. The **Edit Task** form opens, pre-filled with the current values.
3. Change any fields and click **Save Changes**.

> **Tip:** You can edit a task that is currently running. The changes take effect the next time the task starts.

---

## Deleting a task

1. Click **Delete** (red button) on a task card.
2. A danger confirmation dialog appears:
   > "Delete task? This permanently removes '**Task Name**' and cannot be undone."
3. Confirm to delete.

> **Warning:** Deletion is permanent. The task definition is removed immediately and cannot be recovered. Log files on disk are not automatically deleted.

---

## Action errors

If a start, stop, restart, or other action fails, a red error message appears directly below the task's meta row on the card.

---

## Connection states

| Message | Meaning |
|---|---|
| "Connecting to daemon…" | App is starting up; the engine is initialising |
| "Connection lost — showing last known state" | Live connection to the background engine dropped; data shown may be stale |
| "Cannot connect to daemon" | The background engine could not be reached |

If you see a persistent connection error, see [Troubleshooting](./troubleshooting.md).

---

## Related

- [Creating Tasks](./creating-tasks.md) — Full reference for every task form field
- [Viewing Logs](./logs.md) — The live log viewer and log files on disk
- [Auto-restart](./auto-restart.md) — Automatically recover crashed tasks
- [Scheduling (Cron)](./scheduling.md) — Run tasks on a timed schedule
- [Troubleshooting](./troubleshooting.md) — Help with connection errors and other problems
