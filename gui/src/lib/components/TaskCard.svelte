<script lang="ts">
  import { api, type TaskDto, taskId } from '$lib/api/client';
  import { loadTasks } from '$lib/stores/tasks';
  import ConfirmDialog from './ConfirmDialog.svelte';

  let { task, onViewLogs, onEdit } = $props<{
    task: TaskDto;
    onViewLogs: (id: string) => void;
    onEdit: (task: TaskDto) => void;
  }>();

  let loading = $state(false);
  let error = $state<string | null>(null);
  let confirmAction = $state<{ type: 'stop' | 'restart' } | null>(null);
  let cpuUsage = $state<number | null>(null);
  let memUsage = $state<number | null>(null);

  const id = $derived(taskId(task));
  const isRunning = $derived(task.status === 'running' || task.status === 'starting');

  async function action(fn: () => Promise<unknown>) {
    loading = true;
    error = null;
    try {
      await fn();
      await loadTasks();
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  async function handleDelete() {
    if (!confirm(`Delete task "${task.config.description}"?`)) return;
    await action(() => api.tasks.remove(id));
  }

  async function handleStop() {
    await action(() => api.tasks.stop(id));
    confirmAction = null;
  }

  async function handleRestart() {
    await action(() => api.tasks.restart(id));
    confirmAction = null;
  }

  async function updateMetrics() {
    if (isRunning && task.pid) {
      try {
        const stats = await api.tasks.getStats(id);
        cpuUsage = stats.cpu_percent;
        memUsage = stats.memory_bytes;
      } catch (e) {
        // Ignore errors
      }
    }
  }

  $effect(() => {
    if (isRunning) {
      updateMetrics();
      const interval = setInterval(updateMetrics, 5000);
      return () => clearInterval(interval);
    }
  });

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  }
</script>

<div class="task-card glass card" class:running={isRunning} class:crashed={task.status === 'crashed'}>
  <!-- Header row -->
  <div class="card-header">
    <div class="task-info">
      <span class={`status-badge status-${task.status}`}>{task.status}</span>
      <h3 class="task-name">{task.config.description}</h3>
    </div>
    <div class="card-actions">
      <!-- Start / Stop / Restart -->
      {#if isRunning}
        <button class="btn" onclick={() => confirmAction = { type: 'stop' }} disabled={loading}>
          ⏹ Stop
        </button>
        <button class="btn" onclick={() => confirmAction = { type: 'restart' }} disabled={loading}>
          ↺ Restart
        </button>
      {:else}
        <button class="btn btn-primary" onclick={() => action(() => api.tasks.start(id))} disabled={loading}>
          ▶ Start
        </button>
      {/if}
      <button class="btn-icon" title="View Logs" onclick={() => onViewLogs(id)}>
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="3 6 5 6 21 6"/><path d="M19 6l-1 14H6L5 6"/><path d="M10 11v6"/><path d="M14 11v6"/>
        </svg>
      </button>
      <button class="btn-icon" title="Edit" onclick={() => onEdit(task)}>
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
          <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
        </svg>
      </button>
      <button class="btn-icon btn-danger" title="Delete" onclick={handleDelete}>
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="3 6 5 6 21 6"/>
          <path d="M19 6l-1 14H6L5 6"/>
          <path d="M10 11v6"/><path d="M14 11v6"/>
          <path d="M9 6V4h6v2"/>
        </svg>
      </button>
    </div>
  </div>

  <!-- Details row -->
  <div class="card-meta">
    <span class="meta-item" title={task.config.executable}>
      <code>{task.config.executable}</code>
    </span>
    {#if task.pid}
      <span class="meta-item">PID <strong>{task.pid}</strong></span>
    {/if}
    {#if cpuUsage !== null && task.pid}
      <span class="meta-item">
        CPU: <strong>{cpuUsage.toFixed(1)}%</strong>
      </span>
      <span class="meta-item">
        Memory: <strong>{formatBytes(memUsage ?? 0)}</strong>
      </span>
    {/if}
    {#if task.config.run_as_admin}
      <span class="meta-tag admin">Admin</span>
    {/if}
    {#if task.config.auto_restart}
      <span class="meta-tag">Auto-restart</span>
    {/if}
  </div>

  {#if error}
    <p class="card-error">{error}</p>
  {/if}

  {#if confirmAction}
    <ConfirmDialog
      title={confirmAction.type === 'stop' ? 'Stop Task' : 'Restart Task'}
      message={confirmAction.type === 'stop' 
        ? `Stop running task "${task.config.description}"?`
        : `Restart task "${task.config.description}"? This will stop and start it again.`}
      confirmText={confirmAction.type === 'stop' ? 'Stop' : 'Restart'}
      variant={confirmAction.type === 'stop' ? 'warning' : 'danger'}
      onConfirm={confirmAction.type === 'stop' ? handleStop : handleRestart}
      onCancel={() => confirmAction = null}
    />
  {/if}
</div>

<style>
  .task-card {
    margin-bottom: 0.75rem;
    transition: border-color 0.2s;
  }

  .task-card.running {
    border-color: rgba(72 187 120 / 0.3);
  }

  .task-card.crashed {
    border-color: rgba(252 129 129 / 0.3);
  }

  .card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    flex-wrap: wrap;
  }

  .task-info {
    display: flex;
    align-items: center;
    gap: 0.625rem;
  }

  .task-name {
    font-size: 0.9375rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .card-actions {
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }

  .card-meta {
    display: flex;
    align-items: center;
    gap: 0.625rem;
    margin-top: 0.5rem;
    flex-wrap: wrap;
  }

  .meta-item {
    font-size: 0.75rem;
    color: var(--text-secondary);
  }

  .meta-item code {
    font-family: 'Cascadia Code', 'Fira Code', monospace;
    font-size: 0.7rem;
    color: var(--text-secondary);
    max-width: 260px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    display: inline-block;
    vertical-align: bottom;
  }

  .meta-tag {
    font-size: 0.65rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    padding: 0.15rem 0.45rem;
    border-radius: 4px;
    background: var(--bg-surface-hover);
    color: var(--text-muted);
    border: 1px solid var(--border-subtle);
  }

  .meta-tag.admin {
    color: var(--accent);
    border-color: var(--border-accent);
  }

  .card-error {
    margin-top: 0.5rem;
    font-size: 0.75rem;
    color: var(--status-crashed);
  }
</style>
