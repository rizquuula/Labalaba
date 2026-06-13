# Auto-Restart on Crash

Labalaba can automatically restart a task when it crashes, so short-lived failures recover without any action from you.

> **Note:** Auto-restart only runs while the **daemon is running**. The daemon runs whenever the app window is open. To keep auto-restart active after you close the window, enable **Launch at login** in Settings — see [Background Service (Daemon Mode)](./background-service.md).

## How to enable it

1. Open the task form (create a new task or edit an existing one).
2. Switch to the **Advanced** tab.
3. Check the **Auto-restart on crash** checkbox.
4. Save the task.

Once enabled, an **AUTO-RESTART** tag appears on the task card as a reminder.

## When auto-restart triggers

Auto-restart fires **only** when the process exits with a non-zero exit code — meaning a real crash or error exit. A clean exit (exit code 0) is treated as an intentional stop; auto-restart will **not** run again, even if the checkbox is on.

> **Note:** If your program exits with code 0 after finishing its work (a one-shot script, for example), Labalaba will not restart it. That is expected behavior.

## Backoff schedule

Labalaba does not restart instantly every time. It uses an exponential backoff delay to avoid hammering a program that keeps failing:

| Attempt | Wait before restarting |
|---------|------------------------|
| 1st     | 3 seconds              |
| 2nd     | 6 seconds              |
| 3rd     | 12 seconds             |
| 4th     | 24 seconds             |
| 5th     | 48 seconds             |

The delay is capped at 60 seconds, and Labalaba will make a **maximum of 5 consecutive restart attempts**. If all five fail, the task is marked **crashed** and left alone — you will need to investigate and start it manually.

## The 30-second reset rule

The consecutive-attempt counter resets to zero if a run stays up for at least **30 seconds**. This means a task that was stable for a while and then crashes later starts the backoff fresh from 3 seconds rather than counting against its previous failures.

A manual **Start** or **Restart** also clears the counter.

## When to use auto-restart

Auto-restart is useful for:

- Long-running background services that should stay up continuously.
- Programs that occasionally crash due to transient errors (network blips, temporary file locks, etc.).
- Tasks you cannot monitor actively.

## Caveats

- Auto-restart does not help if your program exits cleanly (code 0) — see above.
- After 5 failed attempts the task stays in the **crashed** state. Check the logs for the root cause before restarting it manually.
- Combining auto-restart with a very short-lived task can exhaust the 5 attempts quickly. Consider whether a cron schedule better fits that use case.

## Related

- [Creating Tasks](./creating-tasks.md)
- [Managing Tasks](./managing-tasks.md)
- [Scheduling](./scheduling.md)
- [Background Service (Daemon Mode)](./background-service.md)
- [Logs](./logs.md)
- [Troubleshooting](./troubleshooting.md)
- [Back to Home](./README.md)
