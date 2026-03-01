<script lang="ts">
  import { untrack } from 'svelte';
  import { api, type TaskDto, type TaskRequest, taskId } from '$lib/api/client';
  import { loadTasks } from '$lib/stores/tasks';

  let { task, onClose } = $props<{
    task?: TaskDto;     // undefined = create mode
    onClose: () => void;
  }>();

  const isEdit = $derived(!!task);

  // Form state — untrack() captures the initial prop value intentionally;
  // these are editable form fields, not derived reactive values.
  let name = $state(untrack(() => task?.config.name ?? ''));
  let executable = $state(untrack(() => task?.config.executable ?? ''));
  let argsRaw = $state(untrack(() => (task?.config.arguments ?? []).join(' ')));
  let workingDir = $state(untrack(() => task?.config.working_directory ?? ''));
  let runAsAdmin = $state(untrack(() => task?.config.run_as_admin ?? false));
  let autoRestart = $state(untrack(() => task?.config.auto_restart ?? false));
  let cronExpr = $state(untrack(() => task?.config.schedule?.cron ?? ''));
  let startupDelay = $state(untrack(() => task?.config.startup_delay_ms ?? 0));
  let envRaw = $state(
    untrack(() =>
      Object.entries(task?.config.environment ?? {})
        .map(([k, v]) => `${k}=${v}`)
        .join('\n')
    )
  );

  let saving = $state(false);
  let error = $state<string | null>(null);

  function parseEnv(raw: string): Record<string, string> {
    const env: Record<string, string> = {};
    for (const line of raw.split('\n')) {
      const eq = line.indexOf('=');
      if (eq > 0) {
        env[line.slice(0, eq).trim()] = line.slice(eq + 1).trim();
      }
    }
    return env;
  }

  async function handleSubmit(e: Event) {
    e.preventDefault();
    if (!name.trim() || !executable.trim()) {
      error = 'Name and executable are required';
      return;
    }

    const req: TaskRequest = {
      name: name.trim(),
      executable: executable.trim(),
      arguments: argsRaw.trim() ? argsRaw.trim().split(/\s+/) : [],
      working_directory: workingDir.trim() || undefined,
      environment: parseEnv(envRaw),
      run_as_admin: runAsAdmin,
      auto_restart: autoRestart,
      schedule: cronExpr.trim() ? { cron: cronExpr.trim() } : undefined,
      startup_delay_ms: startupDelay,
      depends_on: [],
    };

    saving = true;
    error = null;
    try {
      if (isEdit && task) {
        await api.tasks.update(taskId(task), req);
      } else {
        await api.tasks.create(req);
      }
      await loadTasks();
      onClose();
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  }

</script>

<div class="modal-backdrop" role="dialog" aria-modal="true">
  <div class="modal glass-strong">
    <div class="modal-header">
      <h2>{isEdit ? 'Edit Task' : 'New Task'}</h2>
      <button class="btn-icon" aria-label="Close" onclick={onClose}>
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
        </svg>
      </button>
    </div>

    <form onsubmit={handleSubmit}>
      <div class="form-grid">
        <div class="form-group full">
          <label for="task-name">Task Name *</label>
          <input id="task-name" class="input" type="text" bind:value={name}
            placeholder="My Application" required />
        </div>

        <div class="form-group full">
          <label for="task-exe">Executable Path *</label>
          <div class="input-row">
            <input id="task-exe" class="input" type="text" bind:value={executable}
              placeholder="C:\path\to\app.exe" required />
          </div>
        </div>

        <div class="form-group full">
          <label for="task-args">Arguments</label>
          <input id="task-args" class="input" type="text" bind:value={argsRaw}
            placeholder="--port 8080 --config config.yaml" />
        </div>

        <div class="form-group full">
          <label for="task-wd">Working Directory</label>
          <input id="task-wd" class="input" type="text" bind:value={workingDir}
            placeholder="C:\path\to\workdir (optional)" />
        </div>

        <div class="form-group full">
          <label for="task-env">Environment Variables <span class="label-hint">(KEY=VALUE per line)</span></label>
          <textarea id="task-env" class="input textarea" rows="3" bind:value={envRaw}
            placeholder="NODE_ENV=production&#10;PORT=8080"></textarea>
        </div>

        <div class="form-group">
          <label for="task-cron">Cron Schedule</label>
          <input id="task-cron" class="input" type="text" bind:value={cronExpr}
            placeholder="0 */6 * * * (optional)" />
        </div>

        <div class="form-group">
          <label for="task-delay">Startup Delay (ms)</label>
          <input id="task-delay" class="input" type="number" min="0" bind:value={startupDelay} />
        </div>

        <div class="toggles">
          <label class="toggle-label">
            <span>Run as Admin</span>
            <label class="toggle">
              <input type="checkbox" bind:checked={runAsAdmin} />
              <span class="toggle-track"></span>
            </label>
          </label>

          <label class="toggle-label">
            <span>Auto-restart on crash</span>
            <label class="toggle">
              <input type="checkbox" bind:checked={autoRestart} />
              <span class="toggle-track"></span>
            </label>
          </label>
        </div>
      </div>

      {#if error}
        <p class="form-error">{error}</p>
      {/if}

      <div class="modal-footer">
        <button type="button" class="btn" onclick={onClose}>Cancel</button>
        <button type="submit" class="btn btn-primary" disabled={saving}>
          {saving ? 'Saving…' : isEdit ? 'Save Changes' : 'Create Task'}
        </button>
      </div>
    </form>
  </div>
</div>

<style>
  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 1.25rem;
  }

  .modal-header h2 {
    font-size: 1.0625rem;
    font-weight: 700;
  }

  .form-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0 1rem;
  }

  .form-group.full { grid-column: 1 / -1; }

  .input-row {
    display: flex;
    gap: 0.5rem;
  }

  .input-row .input { flex: 1; }

  .textarea {
    resize: vertical;
    min-height: 72px;
  }

  .label-hint {
    font-weight: 400;
    color: var(--text-muted);
    font-size: 0.75rem;
  }

  .toggles {
    grid-column: 1 / -1;
    display: flex;
    gap: 1.5rem;
    margin-bottom: 1rem;
  }

  .toggle-label {
    display: flex;
    align-items: center;
    gap: 0.625rem;
    cursor: pointer;
    font-size: 0.875rem;
    color: var(--text-primary);
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
    margin-top: 1.25rem;
    padding-top: 1rem;
    border-top: 1px solid var(--border-subtle);
  }

  .form-error {
    margin-top: 0.5rem;
    font-size: 0.8125rem;
    color: var(--status-crashed);
  }
</style>
