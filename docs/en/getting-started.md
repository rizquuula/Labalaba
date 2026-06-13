# Getting Started

Run your first program with Labalaba in under two minutes.

---

## What is a task?

A **task** is a program you want Labalaba to manage — a server, a script, a background worker, or anything else you can run on your computer. You define it once (name, path, optional arguments), and Labalaba remembers it so you can start, stop, watch, and restart it at any time.

---

## Your first task in three steps

This example runs a simple Python HTTP server, but the steps are the same for any program.

### Step 1 — Open the task form

Click the **New Task** button in the main window (or the **Add your first task** button if your list is empty). The **New Task** form opens.

### Step 2 — Fill in the basics

| Field | What to enter (example) |
|---|---|
| **Description** | `My Python Server` |
| **Executable / Script Path** | `server.py` (or click **Browse** to pick the file) |
| **Python Runner** | `python` (this field appears automatically for `.py` files) |
| **Arguments** | `--port 8080` |

> **Tip:** Click **Browse** to use a file picker instead of typing the path by hand. Labalaba auto-detects the right runner for scripts.

Click **Create Task**. The form closes and your new task appears in the list with a **stopped** badge.

### Step 3 — Start it

On the task card, click **Start**. The badge turns green (**running**) and Labalaba begins tracking its PID, CPU, and memory usage.

---

## Watch the logs

Click **View Logs** on the task card to open the live log viewer. Everything the program writes to stdout or stderr appears here in real time.

Press **Escape** to close the log viewer.

---

## Stop the task

Click **Stop** on the task card. A confirmation dialog appears — click **Stop** again to confirm. The badge returns to **stopped** (gray).

> **Note:** Stopping a task that exits cleanly (exit code 0) is a normal intentional stop — Labalaba does not treat it as a crash.

---

## You're all set

Your task is saved and will always appear in the list. You can start it again anytime with a single click.

---

## Next steps

- [Creating Tasks](./creating-tasks.md) — Every field in the task form explained in full
- [Managing Tasks](./managing-tasks.md) — Search, filter, edit, delete, and understand status badges
- [Viewing Logs](./logs.md) — More about the live log viewer and log files on disk
- [Auto-restart](./auto-restart.md) — Keep a task running automatically if it crashes
