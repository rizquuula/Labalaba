# Logs

Labalaba captures the output of every task and lets you view it live in a built-in log viewer, as well as on disk for later inspection.

## Opening the log viewer

Click **View Logs** on any task card to open the log viewer. It appears as an expandable, resizable panel (roughly 180–380 px tall) that slides in below the task list.

### Viewer header

The header of the log viewer contains:

| Control | Description |
|---------|-------------|
| Task name | Identifies which task you are viewing. |
| **Auto-scroll** checkbox | On by default. Keeps the view scrolled to the latest output. Turn it off to scroll back freely. |
| **Clear** icon | Clears the on-screen display. Does **not** delete the log file on disk. |
| **Close** (×) | Closes the log panel. |

### What you see when it opens

When you first open the log viewer, it loads the most recent ~500 lines from the log file on disk and shows "Loading historical logs…" while it does so. After that, new output streams in live.

If there is no output yet, the viewer shows "Waiting for output…".

### Line format

Each line in the viewer includes:

- A timestamp in **HH:MM:SS** format.
- For stderr output: a red **[ERR]** prefix and red text.
- For stdout output: normal text color.

The viewer uses a monospace font. Long lines wrap rather than overflow.

> **Note:** The viewer keeps up to ~5,000 recent lines in memory. Older lines are dropped as new ones arrive. The full history is always available in the log file on disk.

### Limitations

- **Elevated tasks** (tasks with **Run as Admin** enabled on Windows) run in a separate session. Labalaba cannot capture their output, so the log viewer will be empty or show very limited content for those tasks.
- **Processes recovered after an app restart** may also produce no live log output in the viewer, though their previous output is still on disk.

## Log files on disk

Labalaba writes every task's output to a log file:

```
<data dir>/logs/<task-id>.log
```

The data directory defaults to your platform's per-user data directory (`%APPDATA%\labalaba` on Windows, `~/.local/share/labalaba` on Linux, `~/Library/Application Support/labalaba` on macOS) — not the app's working directory. It can be overridden with the `LABALABA_DATA_DIR` environment variable, or by turning on portable mode (Windows only) in **Settings → Data Location**, which keeps it at `<folder holding the executable>/data` instead. See [Configuration Files](./configuration-files.md) for the full resolution order.

### On-disk line format

```
[timestamp] [stream] line
```

For example:

```
[2026-06-13T09:00:00Z] [stdout] Server listening on port 8080
[2026-06-13T09:00:01Z] [stderr] Warning: config file not found, using defaults
```

### Log rotation

When a log file grows past the **Max File Size (MB)** limit, Labalaba rotates it:

| File name          | Contents                              |
|--------------------|---------------------------------------|
| `<task-id>.log`    | Current (most recent) output          |
| `<task-id>.log.1`  | Previous log (first rotation)         |
| `<task-id>.log.2`  | Older log (second rotation)           |
| …                  | …                                     |
| `<task-id>.log.5`  | Oldest kept log (at default settings) |

Once the maximum number of rotated files (**Max Rotated Files**, default 5) is reached, the oldest file is removed when the next rotation occurs.

### Related settings

The following settings on the **Settings** page control log behavior:

| Setting | Default | Description |
|---------|---------|-------------|
| Log Directory | `<data dir>/logs` | Where log files are stored. |
| Max File Size (MB) | 10 MB | File size that triggers rotation. |
| Max Rotated Files | 5 | How many rotated files to keep. |
| Log Buffer (lines) | — | Number of lines buffered in memory. |

> **Tip:** If disk space is a concern, lower **Max File Size (MB)** and **Max Rotated Files** in Settings.

## Related

- [Admin Elevation](./admin-elevation.md)
- [Managing Tasks](./managing-tasks.md)
- [Settings](./settings.md)
- [Configuration Files](./configuration-files.md)
- [Troubleshooting](./troubleshooting.md)
- [Back to Home](./README.md)
