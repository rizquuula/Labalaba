# Settings

Adjust how Labalaba looks and behaves from the **Settings** panel — every option in one place.

## Opening Settings

Click the **gear icon** in the top bar to open the Settings panel.

## Theme toggle

The **sun/moon button** in the top bar instantly switches between **Light** and **Dark** themes. Your choice is remembered across sessions. You can also change the theme inside Settings under **Appearance**.

## Appearance

| Label | Default | Values | What it does |
|---|---|---|---|
| **Theme** | Dark | Dark, Light | Sets the app's colour theme. Applies immediately. |

## Daemon

These control the internal engine that runs your tasks. Most users never need to touch them.

| Label | Default | Range | What it does |
|---|---|---|---|
| **Daemon Port** | 27015 | 1024–65535 | The local network port the embedded engine uses internally. Change this only if port 27015 is already taken by another program. |
| **Config File Path** | `./tasks.yaml` | Any valid path | Where your task definitions are stored on disk. Relative paths resolve against the data directory. |
| **Log Buffer (lines)** | 5000 | 100–50000 | Maximum number of log lines held in memory per task for the live viewer. Older lines are dropped once the limit is reached. |

## Logs

| Label | Default | Range | What it does |
|---|---|---|---|
| **Log Directory** | `./logs` | Any valid path | Folder where per-task log files are written. Relative paths resolve against the data directory. |
| **Max File Size (MB)** | 10 | 1–1024 | When a task's log file exceeds this size it is rotated (renamed) and a fresh file starts. |
| **Max Rotated Files** | 5 | 0–100 | How many old (rotated) log files to keep per task. Set to 0 to keep none. |

## Notifications

| Label | Default | Values | What it does |
|---|---|---|---|
| **Desktop Notifications** | On | On / Off | Send a desktop alert when a task crashes or stops unexpectedly. |
| **Launch at login** | Off | On / Off | Start Labalaba automatically when you log in to your computer. |

## Updates

| Label | Default | What it does |
|---|---|---|
| **Auto-check for Updates** | On | Check for a new version roughly once a day and show a dialog when one is found. |
| **Check for Updates Now** | — (button) | Immediately checks for a newer version. Shows "Update available: X.Y.Z" with a **View Release** link, or "You're on the latest version (X.Y.Z)" if you're already up to date. |

## Saving your changes

Number inputs are automatically clamped to their valid ranges — you cannot accidentally enter a value outside the allowed range. When you are done, click **Save Settings** (the button briefly shows "Saving…" to confirm). Click **Cancel** to discard all unsaved changes.

> **Tip:** Close any running tasks that write logs before changing **Log Directory** — log files for active tasks will continue writing to the old location until those tasks are restarted.

## Related

- [Notifications & Updates](./notifications-and-updates.md)
- [Configuration Files](./configuration-files.md)
- [Troubleshooting](./troubleshooting.md)
- [Back to home](./README.md)
