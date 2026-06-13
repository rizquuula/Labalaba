# Labalaba — User Guide

**A modern, cross-platform process manager with a glassmorphism desktop UI.**

Labalaba lets you spawn, monitor, and control any program on your computer — server binaries, shell scripts, Python apps, and more — all from one desktop window. Define each program once as a **task**, then start, stop, restart, and watch it live. Tasks are saved, so they're always one click away.

> 🌏 Bahasa Indonesia? [Baca versi Bahasa Indonesia →](../id/README.md)

---

## What can Labalaba do?

| Feature | What it gives you |
|---|---|
| 🚀 **Run anything** | Launch any executable, script, or Python app with custom arguments, environment variables, and working directory |
| 📋 **Saved tasks** | Every task is remembered between restarts — start it again anytime with one click |
| ▶️ **Lifecycle control** | Start, stop, and restart tasks instantly |
| 🔁 **Auto-restart** | Automatically bring a task back up if it crashes (with smart backoff); survives window close in daemon mode |
| ⏰ **Scheduling** | Run tasks on a standard 5-field cron schedule; survives window close in daemon mode |
| 🔗 **Dependencies & delays** | Stagger startups with a delay and order tasks by dependency |
| 🛡️ **Admin elevation** | Run a task with Administrator privileges (Windows) |
| 📡 **Live logs** | Watch stdout/stderr stream in a real-time terminal viewer |
| 📊 **Live stats** | See each task's status, PID, CPU %, and memory usage |
| 🔔 **Notifications** | Get a desktop alert when a task crashes or stops unexpectedly |
| 🌓 **Light / Dark theme** | A polished glassmorphism interface, your way |
| ⬆️ **Auto-updates** | Get notified when a new version is available |

---

## Documentation map

### 🏁 Getting started
1. [Installation](./installation.md) — Download and install on Windows, macOS, or Linux (or build from source)
2. [Getting Started](./getting-started.md) — Create and run your very first task in three steps

### 📦 Working with tasks
3. [Creating Tasks](./creating-tasks.md) — The complete task form reference: every field explained
4. [Managing Tasks](./managing-tasks.md) — Status, start/stop/restart, editing, deleting, search, and live stats

### ⚙️ Features in depth
5. [Auto-restart](./auto-restart.md) — Keep crashed tasks running automatically
6. [Scheduling (Cron)](./scheduling.md) — Run tasks on a schedule
7. [Background Service (Daemon Mode)](./background-service.md) — Keep schedules and auto-restart running after you close the window
8. [Dependencies & Startup Delay](./dependencies.md) — Control start order and timing
9. [Admin Elevation](./admin-elevation.md) — Run tasks as Administrator
10. [Viewing Logs](./logs.md) — The live log viewer plus log files on disk

### 🔧 Configuration & help
11. [Settings](./settings.md) — Every setting explained, plus themes
12. [Notifications & Updates](./notifications-and-updates.md) — Desktop alerts and staying up to date
13. [Configuration Files](./configuration-files.md) — `tasks.yaml`, `settings.yaml`, and the data directory
14. [Troubleshooting](./troubleshooting.md) — Fixes for common problems
15. [FAQ](./faq.md) — Frequently asked questions

---

## A 60-second tour

1. **Open Labalaba.** The background daemon starts automatically — there's nothing else to install or run. The app lives in your system tray; closing the window hides it rather than quitting.
2. **Click "New Task"**, give it a name, and pick the program to run (a binary, a `.py`/`.sh`/`.ps1` script, anything).
3. **Hit ▶ Start.** Your program launches, and its output streams live into the log viewer.
4. Labalaba keeps watching it — showing **CPU**, **memory**, and **status** — and can restart it automatically if it crashes.

Ready? Start with [Installation →](./installation.md)

---

<div align="center">

Made with ❤️ in Rust · [Report a bug](https://github.com/rizquuula/labalaba/issues) · [Request a feature](https://github.com/rizquuula/labalaba/issues)

</div>
