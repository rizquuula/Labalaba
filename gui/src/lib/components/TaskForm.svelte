<script lang="ts">
  import { untrack } from 'svelte';
  import { focusTrap } from '$lib/actions/focusTrap';
  import { open } from '@tauri-apps/plugin-dialog';
  import { api, type TaskDto, type TaskRequest, taskId } from '$lib/api/client';
  import { loadTasks } from '$lib/stores/tasks';

  let { task, onClose } = $props<{
    task?: TaskDto;
    onClose: () => void;
  }>();

  const isEdit = $derived(!!task);

  let name = $state(untrack(() => task?.config.description ?? ''));
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

  let scriptType = $state<'binary' | 'python'>(untrack(() => task?.config.runner_prefix ? 'python' : 'binary'));
  let runnerPrefix = $state(untrack(() => task?.config.runner_prefix ?? 'python'));
  let customRunner = $state('');
  const runnerPresets = ['python', 'pythonw', 'uv run', 'pipenv run python', 'poetry run python'];

  let activeTab = $state<'basic' | 'advanced'>('basic');
  let saving = $state(false);
  let error = $state<string | null>(null);

  const modalHeadingId = 'task-form-heading';

  function switchTab(tab: 'basic' | 'advanced') {
    activeTab = tab;
  }

  function handleTabKeydown(e: KeyboardEvent) {
    if (e.key === 'ArrowLeft') {
      e.preventDefault();
      switchTab('basic');
      (document.getElementById('tab-basic') as HTMLElement | null)?.focus();
    } else if (e.key === 'ArrowRight') {
      e.preventDefault();
      switchTab('advanced');
      (document.getElementById('tab-advanced') as HTMLElement | null)?.focus();
    }
  }

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

  function detectOs(): 'windows' | 'macos' | 'linux' {
    const ua = navigator.userAgent;
    if (ua.includes('Windows')) return 'windows';
    if (ua.includes('Mac')) return 'macos';
    return 'linux';
  }

  // Filters are platform-aware: Windows executables carry extensions, but native
  // binaries on macOS/Linux usually have none, so we show all files there rather
  // than hide them behind an extension filter.
  function executableFilters(): { name: string; extensions: string[] }[] | undefined {
    if (scriptType === 'python') {
      return [{ name: 'Python Script', extensions: ['py', 'pyw'] }];
    }
    switch (detectOs()) {
      case 'windows':
        return [{ name: 'Executable', extensions: ['exe', 'bat', 'cmd', 'ps1'] }];
      default:
        // macOS / Linux: no filter so extension-less binaries remain selectable.
        return undefined;
    }
  }

  async function pickExecutable() {
    const selected = await open({
      multiple: false,
      filters: executableFilters()
    });
    if (!selected || typeof selected !== 'string') return;

    const norm = selected.replace(/\\/g, '/');
    const slash = norm.lastIndexOf('/');
    const dir = slash >= 0 ? selected.slice(0, slash) : '';
    const fileName = slash >= 0 ? norm.slice(slash + 1) : norm;
    const dot = fileName.lastIndexOf('.');
    const ext = dot >= 0 ? fileName.slice(dot + 1).toLowerCase() : '';
    const os = detectOs();

    // Shell/PowerShell/batch scripts can't be exec'd directly — the OS needs a
    // shebang (or file association) we can't assume — so launch them via an
    // installed interpreter, passing the bare filename relative to the script's
    // own directory as the working dir.
    const unixKinds: Record<string, 'sh' | 'bash' | 'zsh'> = {
      sh: 'sh', command: 'sh', bash: 'bash', zsh: 'zsh'
    };
    const winKinds: Record<string, 'ps1' | 'bat'> = {
      ps1: 'ps1', cmd: 'bat', bat: 'bat'
    };
    const kind = os === 'windows' ? (winKinds[ext] ?? null) : (unixKinds[ext] ?? null);

    if (kind) {
      const interpreter = await api.system.detectInterpreter(kind);
      if (interpreter) {
        executable = interpreter;
        workingDir = dir;
        argsRaw =
          kind === 'ps1' ? `-ExecutionPolicy Bypass -File ${fileName}`
          : kind === 'bat' ? `/c ${fileName}`
          : fileName;
        return;
      }
      // No interpreter installed — fall through to selecting the script directly.
    }

    executable = selected;
    if (!workingDir) {
      workingDir = dir;
    }
  }

  async function pickWorkingDir() {
    const selected = await open({
      multiple: false,
      directory: true,
      title: 'Select Working Directory',
    });
    if (selected && typeof selected === 'string') {
      workingDir = selected;
    }
  }

  async function handleSubmit(e: Event) {
    e.preventDefault();
    if (!name.trim()) {
      error = 'Task name is required';
      return;
    }
    if (!executable.trim()) {
      error = 'Executable path is required';
      return;
    }

    // Handle runner prefix for Python scripts
    const finalRunnerPrefix = scriptType === 'python'
      ? (runnerPrefix === 'custom' ? customRunner : runnerPrefix)
      : undefined;

    const req: TaskRequest = {
      description: name.trim(),
      executable: executable.trim(),
      arguments: argsRaw.trim() ? argsRaw.trim().split(/\s+/) : [],
      working_directory: workingDir.trim() || undefined,
      environment: parseEnv(envRaw),
      run_as_admin: runAsAdmin,
      auto_restart: autoRestart,
      schedule: cronExpr.trim() ? { cron: cronExpr.trim() } : undefined,
      startup_delay_ms: startupDelay,
      depends_on: [],
      runner_prefix: finalRunnerPrefix,
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

<div class="modal-backdrop" role="dialog" aria-modal="true" aria-labelledby={modalHeadingId} use:focusTrap={{ onClose }}>
  <div class="modal glass-strong">
    <div class="modal-header">
      <h2 id={modalHeadingId}>{isEdit ? 'Edit Task' : 'New Task'}</h2>
      <button class="btn-icon" aria-label="Close" onclick={onClose} disabled={saving}>
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
        </svg>
      </button>
    </div>

    <div class="script-type-selector">
      <label class="radio-group">
        <input type="radio" name="scriptType" value="binary" bind:group={scriptType} />
        <span>Binary File</span>
      </label>
      <label class="radio-group">
        <input type="radio" name="scriptType" value="python" bind:group={scriptType} />
        <span>Python Script</span>
      </label>
    </div>

    <div class="tabs" role="tablist">
      <button
        id="tab-basic"
        class="tab"
        class:active={activeTab === 'basic'}
        role="tab"
        aria-selected={activeTab === 'basic'}
        aria-controls="tabpanel-basic"
        tabindex={activeTab === 'basic' ? 0 : -1}
        onclick={() => switchTab('basic')}
        onkeydown={handleTabKeydown}
      >
        Basic
      </button>
      <button
        id="tab-advanced"
        class="tab"
        class:active={activeTab === 'advanced'}
        role="tab"
        aria-selected={activeTab === 'advanced'}
        aria-controls="tabpanel-advanced"
        tabindex={activeTab === 'advanced' ? 0 : -1}
        onclick={() => switchTab('advanced')}
        onkeydown={handleTabKeydown}
      >
        Advanced
      </button>
    </div>

    <form onsubmit={handleSubmit}>
      {#if activeTab === 'basic'}
        <div id="tabpanel-basic" role="tabpanel" aria-labelledby="tab-basic" tabindex="0">
          <div class="form-group full">
            <label for="task-name">Description *</label>
            <input id="task-name" class="input" type="text" bind:value={name}
              placeholder="My Application" required />
          </div>

          {#if scriptType === 'python'}
            <div class="form-group full">
              <label for="runner-prefix">Python Runner</label>
              <select id="runner-prefix" class="input" bind:value={runnerPrefix}>
                {#each runnerPresets as preset}
                  <option value={preset}>{preset}</option>
                {/each}
                <option value="custom">Custom...</option>
              </select>
            </div>

            {#if runnerPrefix === 'custom'}
              <div class="form-group full">
                <label for="custom-runner">Custom Runner Command</label>
                <input id="custom-runner" class="input" type="text" bind:value={customRunner}
                  placeholder="e.g., uv run or /home/user/.venv/bin/python" />
                <small class="form-hint">Enter the full runner command (e.g., "uv run", "/home/user/.venv/bin/python")</small>
              </div>
            {/if}

            <div class="form-group full">
              <label for="python-script">Python Script Path *</label>
              <div class="input-row">
                <input id="python-script" class="input" type="text" bind:value={executable}
                  placeholder="C:\path\to\script.py" required />
                <button type="button" class="btn btn-secondary" onclick={pickExecutable}>
                  Browse
                </button>
              </div>
            </div>
          {:else}
            <div class="form-group full">
              <label for="task-exe">Executable Path *</label>
              <div class="input-row">
                <input id="task-exe" class="input" type="text" bind:value={executable}
                  placeholder="C:\path\to\app.exe" required />
                <button type="button" class="btn btn-secondary" onclick={pickExecutable}>
                  Browse
                </button>
              </div>
            </div>
          {/if}

          <div class="form-group full">
            <label for="task-args">Arguments</label>
            <input id="task-args" class="input" type="text" bind:value={argsRaw}
              placeholder="--port 8080 --config config.yaml" />
          </div>
        </div>
      {:else}
        <div id="tabpanel-advanced" role="tabpanel" aria-labelledby="tab-advanced" tabindex="0">
          <div class="form-grid">
            <div class="form-group full">
              <label for="task-wd">Working Directory</label>
              <div class="input-row">
                <input id="task-wd" class="input" type="text" bind:value={workingDir}
                  placeholder="C:\path\to\workdir" />
                <button type="button" class="btn btn-secondary" onclick={pickWorkingDir}>
                  Browse
                </button>
              </div>
            </div>

            <div class="form-group full">
              <label for="task-env">Environment Variables <span class="label-hint">(KEY=VALUE per line)</span></label>
              <textarea id="task-env" class="input textarea" rows="3" bind:value={envRaw}
                placeholder="NODE_ENV=production&#10;PORT=8080"></textarea>
              <small class="form-hint">One KEY=VALUE per line; values may contain =; lines without = are ignored</small>
            </div>

            <div class="form-group">
              <label for="task-cron">Cron Schedule</label>
              <input id="task-cron" class="input" type="text" bind:value={cronExpr}
                placeholder="0 */6 * * * (optional)" />
              <small class="form-hint">Standard 5-field cron, e.g. 0 */6 * * * — leave blank to run manually</small>
            </div>

            <div class="form-group">
              <label for="task-delay">Startup Delay (ms)</label>
              <input id="task-delay" class="input" type="number" min="0" bind:value={startupDelay} />
              <small class="form-hint">In milliseconds (5000 = 5 seconds)</small>
            </div>

            <div class="toggles">
              <div class="toggle-label">
                <span>Run as Admin</span>
                <label class="toggle">
                  <input type="checkbox" bind:checked={runAsAdmin} />
                  <span class="toggle-track"></span>
                </label>
              </div>

              <div class="toggle-label">
                <span>Auto-restart on crash</span>
                <label class="toggle">
                  <input type="checkbox" bind:checked={autoRestart} />
                  <span class="toggle-track"></span>
                </label>
              </div>
            </div>
          </div>
        </div>
      {/if}

      {#if error}
        <p class="form-error">{error}</p>
      {/if}

      <div class="modal-footer">
        <button type="button" class="btn" onclick={onClose} disabled={saving}>Cancel</button>
        <button type="submit" class="btn btn-primary" disabled={saving}>
          {saving ? (isEdit ? 'Saving…' : 'Creating…') : isEdit ? 'Save Changes' : 'Create Task'}
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
    margin-bottom: 1rem;
  }

  .modal-header h2 {
    font-size: 1.0625rem;
    font-weight: 700;
  }

  .script-type-selector {
    display: flex;
    gap: 1.5rem;
    margin-bottom: 1rem;
    padding: 0.75rem;
    background: var(--bg-surface-hover);
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-subtle);
  }

  .radio-group {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    cursor: pointer;
    font-size: 0.875rem;
    color: var(--text-primary);
  }

  .tabs {
    display: flex;
    gap: 0.25rem;
    margin-bottom: 1.25rem;
    border-bottom: 1px solid var(--border-subtle);
  }

  .tab {
    background: none;
    border: none;
    padding: 0.5rem 1rem;
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--text-muted);
    cursor: pointer;
    border-bottom: 2px solid transparent;
    margin-bottom: -1px;
    transition: color 0.15s, border-color 0.15s;
  }

  .tab:hover {
    color: var(--text-primary);
  }

  .tab.active {
    color: var(--text-primary);
    border-bottom-color: var(--accent);
  }

  .form-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0 1rem;
  }

  @media (max-width: 460px) {
    .form-grid {
      grid-template-columns: 1fr;
    }
  }

  .form-group.full { grid-column: 1 / -1; }

  .input-row {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
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

  .form-hint {
    display: block;
    margin-top: 0.25rem;
    font-size: 0.75rem;
    color: var(--text-muted);
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
