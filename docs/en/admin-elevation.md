# Admin Elevation

Some programs need elevated privileges to run — for example, to bind to a low-numbered port, access protected system resources, or modify system configuration. Labalaba provides the **Run as Admin** toggle to handle this on Windows.

## How to enable it

1. Open the task form (create a new task or edit an existing one).
2. Switch to the **Advanced** tab.
3. Enable the **Run as Admin** toggle.
4. Save the task.

An **ADMIN** tag will appear on the task card when this option is on.

## Behavior by platform

### Windows

When **Run as Admin** is on, Labalaba launches the program through a UAC (User Account Control) prompt. Windows will ask you to confirm the elevation before the process starts.

> **Warning:** Because the elevated process runs in a separate Windows session, Labalaba **cannot capture its stdout or stderr**. This means the live log viewer will be empty or show very limited output for elevated tasks. If you need to see output from an elevated program, redirect its output to a file within the program itself.

### macOS and Linux

The **Run as Admin** toggle has **no effect** on macOS or Linux. If your task needs elevated privileges on these platforms, use `sudo` inside your command or runner directly.

For example, instead of relying on the toggle, set your executable or command to include `sudo`:

```
sudo /path/to/your/program --your-flags
```

> **Note:** Using `sudo` in a command may still prompt for a password in the terminal where Labalaba was launched, depending on your system's `sudo` configuration.

## When to use admin elevation

Use **Run as Admin** when your program genuinely requires it — for example:

- Binding to a port below 1024 on Windows.
- Installing or managing Windows services.
- Accessing protected system directories.

## Security caution

Running programs with elevated privileges carries risk. A bug or malicious payload in an elevated process can affect your entire system. Only enable **Run as Admin** for programs you trust and that truly require it. Prefer running tasks without elevation whenever possible.

## Related

- [Creating Tasks](./creating-tasks.md)
- [Logs](./logs.md)
- [Troubleshooting](./troubleshooting.md)
- [FAQ](./faq.md)
- [Back to Home](./README.md)
