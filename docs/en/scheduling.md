# Scheduling

You can tell Labalaba to start a task automatically on a recurring schedule using a cron expression.

## Setting a schedule

1. Open the task form (create a new task or edit an existing one).
2. Switch to the **Advanced** tab.
3. Fill in the **Cron Schedule** field with a valid expression (see format below).
4. Save the task.

Leave the field blank if you want to start the task manually only.

> **Note:** All schedule times are evaluated in **UTC**. Keep this in mind if you want a task to run at a specific local time.

> **Important:** Scheduled runs only fire while the **daemon is running**. The daemon runs whenever the app window is open. To keep schedules firing after you close the window, enable **Launch at login** in Settings — see [Background Service (Daemon Mode)](./background-service.md).

## Cron expression format

Labalaba uses standard **5-field** cron format — the same format used by Unix cron and most cron tools.

The fields, in order:

| Position | Field         | Allowed values           |
|----------|---------------|--------------------------|
| 1        | Minute        | 0 – 59                   |
| 2        | Hour          | 0 – 23                   |
| 3        | Day of month  | 1 – 31                   |
| 4        | Month         | 1 – 12                   |
| 5        | Day of week   | 0 – 6 (0 = Sunday)       |

Standard cron syntax applies: `*` means "every value", `*/n` means "every n units", ranges like `1-5`, and lists like `1,3,5`.

## Examples

| Cron expression  | Meaning                              |
|------------------|--------------------------------------|
| `0 */6 * * *`    | Every 6 hours                        |
| `0 0 * * *`      | Every day at midnight (UTC)          |
| `*/30 * * * *`   | Every 30 minutes                     |
| `0 9 * * 1-5`    | 09:00 UTC every weekday (Mon – Fri)  |
| `0 0 * * 0`      | Every Sunday at midnight (UTC)       |

> **Tip:** If you want to run something every day at a specific local time, convert your local time to UTC first. For example, 09:00 UTC+7 is `0 2 * * *` (02:00 UTC).

## Related

- [Creating Tasks](./creating-tasks.md)
- [Auto-Restart on Crash](./auto-restart.md)
- [Background Service (Daemon Mode)](./background-service.md)
- [Dependencies](./dependencies.md)
- [Settings](./settings.md)
- [Back to Home](./README.md)
