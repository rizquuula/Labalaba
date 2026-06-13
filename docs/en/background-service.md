# Background Service (Daemon Mode)

Labalaba is built around a background engine called the **daemon** — a separate process that does the actual work: spawning tasks, watching for crashes, firing cron schedules, and streaming logs. The GUI window is a thin client that connects to it over a local, token-secured HTTP/WebSocket connection on `127.0.0.1`.

Understanding the two modes the daemon can run in will help you decide how to configure Labalaba for your workflow.

---

## Two modes

| Mode | How to enable | What survives closing the window |
|---|---|---|
| **Daemon mode** | Settings → **Launch at login** ON | Everything: cron schedules fire, auto-restart keeps watching, the daemon keeps running |
| **Session only** *(default)* | **Launch at login** OFF | Only already-running task **processes** keep running; cron and auto-restart stop with the daemon |

> **Note:** In either mode, task processes you have already started are OS processes with their own PIDs. Stopping the daemon or closing the window does not kill them — they keep running. Labalaba will re-discover them on the next start by checking their PIDs.

---

## How to enable daemon mode

1. Open the **Settings** panel (gear icon in the top bar).
2. Under **Notifications**, toggle **Launch at login** to **On**.
3. Click **Save Settings**.

From that point on, the daemon starts automatically at login and keeps running after you close the window.

To turn it off, flip the same toggle back to **Off** and save. The daemon will no longer autostart; it will stop when you quit the app.

### What gets registered per OS

No administrator or root access is needed — Labalaba registers a **user-level** autostart entry that only runs for your account and stops when you log out.

| OS | Mechanism |
|---|---|
| **Linux** | A `systemd` user service (`~/.config/systemd/user/`) |
| **macOS** | A Login Item / LaunchAgent (`~/Library/LaunchAgents/`) |
| **Windows** | A startup registry entry under `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` |

---

## System tray

Labalaba lives in your system tray. Closing the main window **does not quit the app** — it hides the window while the tray icon remains. This lets the daemon keep running without a window taking up space.

- **Reopen the window:** click the tray icon.
- **Quit fully:** right-click the tray icon and choose **Quit** (or use **Quit** from the tray menu). This stops the daemon and exits the app.
- **Single instance:** launching Labalaba again when it is already running (even with the window hidden) focuses the existing window rather than starting a second copy.

---

## What survives what

| Event | Already-running task processes | Cron schedules & auto-restart |
|---|---|---|
| Close window (daemon mode ON) | Running | Running |
| Close window (daemon mode OFF) | Running | Stopped |
| Quit from tray | Running | Stopped |
| Computer restart (daemon mode ON) | Stopped | Resume at next login |
| Computer restart (daemon mode OFF) | Stopped | Stopped |

> **Note:** Task processes are not automatically started on reboot regardless of mode. To start a task on login, combine daemon mode with a cron schedule of `@reboot` — or use your OS's own startup mechanism.

---

## Related

- [Scheduling (Cron)](./scheduling.md)
- [Auto-Restart on Crash](./auto-restart.md)
- [Settings](./settings.md)
- [Configuration Files](./configuration-files.md)
- [Back to Home](./README.md)
