# Troubleshooting

A practical guide to common problems and how to fix them.

---

## Task won't start

**Possible causes and fixes:**

- **Wrong path** — Open the task's edit form and double-check the **Executable / Script Path**. Copy-paste the path from your file manager to avoid typos.
- **Missing permissions** — On macOS or Linux, the file may not be executable. Open a terminal and run `chmod +x /path/to/your/program`, then try again.
- **Wrong working directory** — If your program expects to find files relative to its location, make sure **Working Directory** is set to the correct folder. Leave it blank to use the app's own directory.

---

## Task immediately shows "crashed"

A task shows **crashed** when it exits with a non-zero (error) code.

- Check the **Logs** panel for any error message the program printed before it quit.
- Review the **arguments** you passed — a typo or a missing required flag will often cause an instant exit.
- Check that any **environment variables** the program needs are set correctly in the task's **Environment** section.

---

## No logs appear

Three common reasons:

1. **The task is Run as Admin (Windows)** — When a task is elevated via UAC, its output cannot be captured by Labalaba. The logs panel will be empty. This is a Windows limitation. See [Admin Elevation](./admin-elevation.md).
2. **The task was recovered after an app restart** — If Labalaba was closed and reopened, it can detect that a process is still running by its PID, but it cannot reach back and recapture output that was printed before the app started. New output will appear as the task continues running.
3. **The task simply hasn't printed anything yet** — Some programs buffer their output or only log after certain events. Wait a moment, or check the program's own documentation.

---

## "Cannot connect to daemon" or port conflict

The embedded engine uses a local port (default **27015**) for internal communication. If another program on your computer is already using that port, Labalaba cannot start its engine.

**Fix:**

1. Open **Settings** (gear icon).
2. Under **Daemon**, change **Daemon Port** to a different value (any number from 1024 to 65535 that isn't in use).
3. Click **Save Settings** and restart Labalaba.

---

## A scheduled task never fires

The **Cron Schedule** field uses a **6-field** format that includes seconds:

```
second  minute  hour  day-of-month  month  day-of-week
```

The placeholder shown in the UI has 5 fields (a common format), but Labalaba requires the 6-field version. For example, to run every day at 9 AM UTC:

```
0 0 9 * * *
```

All times are interpreted as UTC. See [Scheduling (Cron)](./scheduling.md) for a full reference.

---

## Hand-edited tasks.yaml or settings.yaml changes get overwritten

Labalaba writes to these files whenever you save changes in the UI. If the app is open at the same time you edit the file, it may overwrite your edits on the next save.

**Fix:** Always close Labalaba before editing either file by hand, then reopen it. See [Configuration Files](./configuration-files.md).

---

## Update check fails

If clicking **Check for Updates Now** returns an error:

- Make sure you have an active internet connection.
- Check that your firewall or security software allows Labalaba to make outbound HTTPS requests to `github.com`.
- Try again later — GitHub may be temporarily unavailable.

You can always download the latest release manually from `https://github.com/rizquuula/labalaba/releases`.

---

## A task keeps restarting then stops

Auto-restart only triggers on a non-zero exit. If a task keeps crashing and restarting, Labalaba backs off gradually (3 → 6 → 12 → 24 → 48 seconds, capped at 60) and stops after **5 consecutive failed attempts**, leaving the task as **crashed**.

The counter resets automatically if the task stays up for at least 30 seconds. A manual **Start** or **Restart** also clears the counter.

To understand why the task keeps crashing, check its logs for error output. See [Auto-restart](./auto-restart.md) for full details.

---

## Related

- [Auto-restart](./auto-restart.md)
- [Scheduling (Cron)](./scheduling.md)
- [Admin Elevation](./admin-elevation.md)
- [Configuration Files](./configuration-files.md)
- [Settings](./settings.md)
- [FAQ](./faq.md)
- [Back to home](./README.md)
