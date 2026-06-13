# Dependencies

When you run a stack of tasks together, you often need them to start in a specific order — for example, a database before an API server. Labalaba gives you two tools to control this: **Startup Delay** and **depends_on**.

## Startup Delay

**Startup Delay (ms)** is available in the **Advanced** tab of the task form.

When you start a task that has a delay configured, Labalaba waits that many milliseconds before actually launching the process. While waiting, the task shows the **starting** status. Once the delay passes, the process starts normally.

| Delay value | Actual wait |
|-------------|-------------|
| `0`         | No delay (default) |
| `1000`      | 1 second    |
| `5000`      | 5 seconds   |
| `10000`     | 10 seconds  |

> **Tip:** Use the hint text as a reminder — the field shows "In milliseconds (5000 = 5 seconds)".

A startup delay is a simple, reliable way to give a service time to come up before the next one tries to connect to it.

## Dependencies (`depends_on`)

The `depends_on` setting tells Labalaba which other tasks should be up before this one starts. It works alongside the startup delay to give you precise ordering.

> **Note:** There is no **Dependencies** field in the task form. You set `depends_on` by editing the `tasks.yaml` configuration file directly.

Add a `depends_on` list to a task in `tasks.yaml`, referencing the **id** values of the tasks that must start first:

```yaml
tasks:
  - id: "db-task-id"
    description: "Database"
    # ... other fields ...

  - id: "api-task-id"
    description: "API Server"
    startup_delay_ms: 5000
    depends_on: ["db-task-id"]
    # ... other fields ...
```

In the example above, the **API Server** task will not start until **Database** is up, and it will also wait an additional 5 seconds (via `startup_delay_ms`) before launching, giving the database time to finish initializing.

## Sequencing a small stack

Here is a pattern for bringing up a three-task stack in order:

```yaml
tasks:
  - id: "database"
    description: "Database"
    # No delay — starts immediately

  - id: "cache"
    description: "Cache"
    startup_delay_ms: 3000
    depends_on: ["database"]
    # Waits for database, then waits 3 more seconds

  - id: "api"
    description: "API Server"
    startup_delay_ms: 5000
    depends_on: ["database", "cache"]
    # Waits for both database and cache, then waits 5 more seconds
```

> **Warning:** Editing `tasks.yaml` directly requires care. Make a backup before editing, and ensure your YAML indentation is correct. See [Configuration Files](./configuration-files.md) for guidance.

## Related

- [Creating Tasks](./creating-tasks.md)
- [Managing Tasks](./managing-tasks.md)
- [Scheduling](./scheduling.md)
- [Configuration Files](./configuration-files.md)
- [Back to Home](./README.md)
