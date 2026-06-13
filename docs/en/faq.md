# FAQ

Answers to the most common questions about Labalaba.

---

**Is Labalaba free and open source?**

Yes. Labalaba is MIT licensed and free to use, modify, and distribute. The source code is available on GitHub at `https://github.com/rizquuula/labalaba`.

---

**What platforms does Labalaba support?**

Windows (x64), Linux (x64, AppImage), and macOS (Intel and Apple Silicon).

---

**Do I need to run a separate server or daemon?**

No. The engine that monitors and controls your tasks is embedded directly inside the desktop app and starts automatically when you open it. There is nothing else to install or run.

---

**Where are my tasks and logs stored?**

By default they live in the app's data directory (the same folder as the Labalaba binary). You will find `tasks.yaml` (your task definitions), `settings.yaml` (your preferences), and a `logs/` folder (per-task log files). See [Configuration Files](./configuration-files.md) for details.

---

**Can I move my setup to another computer?**

Yes. Copy `tasks.yaml` and `settings.yaml` to the same relative location on the new machine (or wherever your data directory is). Your tasks and settings will be restored the next time you open Labalaba.

---

**If I close the window, do my tasks keep running?**

Closing the Labalaba window exits the app, which also stops the embedded engine. Your tasks will no longer be monitored. When you reopen Labalaba, it checks which of your previously-running processes are still alive by PID and marks them **running** or **crashed** accordingly — but it cannot recapture any output they produced while the app was closed.

---

**What is the difference between "stopped" and "crashed"?**

- **Stopped** — the task is not running because you stopped it manually, or it has never been started.
- **Crashed** — the task exited on its own with an error (non-zero exit code). A program that exits cleanly with code 0 is shown as **stopped**, not crashed.

---

**Why are the logs empty for a "Run as Admin" task?**

On Windows, UAC-elevated processes run in a separate security context that prevents Labalaba from capturing their output. This is a Windows limitation — nothing can be done about it from within the app. On macOS and Linux, the **Run as Admin** toggle has no effect; use `sudo` in your command arguments instead. See [Admin Elevation](./admin-elevation.md).

---

**Can I run a Python script without changing my PATH?**

Yes. When you set a task's **Executable / Script Path** to a file ending in `.py` or `.pyw`, a **Python Runner** dropdown appears. Choose from `python`, `pythonw`, `uv run`, `pipenv run python`, `poetry run python`, or enter a custom command. Labalaba will use that interpreter to launch your script so you don't have to add anything to your system PATH.

---

**Can I run a task on a schedule?**

Yes. Set the **Cron Schedule** field when creating or editing a task. Use a 6-field cron string (seconds, minutes, hours, day-of-month, month, day-of-week) and note that times are UTC. See [Scheduling (Cron)](./scheduling.md) for examples and the full field reference.

---

**How do I start tasks automatically when my computer boots?**

Turn on **Launch at login** in Settings (Settings → **Notifications** section → **Launch at login**). Labalaba will open automatically when you log in, and any tasks with **Auto-restart** enabled will resume as usual.

---

## Related

- [Getting Started](./getting-started.md)
- [Creating Tasks](./creating-tasks.md)
- [Managing Tasks](./managing-tasks.md)
- [Auto-restart](./auto-restart.md)
- [Scheduling (Cron)](./scheduling.md)
- [Admin Elevation](./admin-elevation.md)
- [Configuration Files](./configuration-files.md)
- [Troubleshooting](./troubleshooting.md)
- [Back to home](./README.md)
