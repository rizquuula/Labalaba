# Notifications & Updates

Stay informed when tasks need attention and keep Labalaba itself up to date.

---

## Desktop notifications

Labalaba can send a native desktop alert any time a task crashes or stops unexpectedly, so you don't have to keep the window open to know when something goes wrong.

**To enable or disable notifications:**

1. Open Settings (gear icon in the top bar).
2. Go to the **Notifications** section.
3. Toggle **Desktop Notifications** on or off.
4. Click **Save Settings**.

When notifications are on, you will receive an alert whenever a task:

- Exits with a non-zero (error) code — the task shows as **crashed**.
- Stops in any other unexpected way.

> **Note:** You will not get an alert when you stop a task manually — notifications are for unexpected stops only.

---

## Staying up to date

### Automatic update checks

When **Auto-check for Updates** is turned on (the default), Labalaba quietly checks for a new version roughly once every 24 hours. You don't have to do anything.

When a newer version is found, a **New Version Available** dialog appears showing:

- Your current version → the available version.
- Release notes for the new version (when provided).
- Two buttons:
  - **Remind Me Later** — dismisses the dialog; Labalaba will surface it again at the next check.
  - **Download Update** — opens the GitHub release page in your browser at `https://github.com/rizquuula/labalaba/releases` so you can download the installer for your platform.

### Checking for updates manually

You can check at any time without waiting for the automatic interval:

1. Open Settings (gear icon in the top bar).
2. Go to the **Updates** section.
3. Click **Check for Updates Now**.

The result appears inline:

- If an update is available: "Update available: X.Y.Z" with a **View Release** link.
- If you are already on the latest release: "You're on the latest version (X.Y.Z)".

### Turning off automatic checks

1. Open Settings.
2. Under **Updates**, toggle **Auto-check for Updates** off.
3. Click **Save Settings**.

You can still check manually at any time using **Check for Updates Now**.

> **Tip:** If the update check fails, make sure you have an active internet connection and that your firewall allows Labalaba to reach `github.com`. See [Troubleshooting](./troubleshooting.md) for more detail.

---

## Related

- [Settings](./settings.md)
- [Troubleshooting](./troubleshooting.md)
- [Back to home](./README.md)
