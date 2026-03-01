<script lang="ts">
  import { tasks, tasksLoading, tasksError } from '$lib/stores/tasks';
  import TaskCard from './TaskCard.svelte';
  import type { TaskDto } from '$lib/api/client';

  let { onViewLogs, onEdit, onAddNew } = $props<{
    onViewLogs: (id: string) => void;
    onEdit: (task: TaskDto) => void;
    onAddNew: () => void;
  }>();
</script>

<section class="task-list">
  <div class="list-header">
    <h2 class="section-title">Tasks</h2>
    <button class="btn btn-primary" onclick={onAddNew}>
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
        <line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>
      </svg>
      New Task
    </button>
  </div>

  {#if $tasksLoading && $tasks.length === 0}
    <div class="empty-state">
      <div class="spinner"></div>
      <p>Connecting to daemon…</p>
    </div>
  {:else if $tasksError && $tasks.length === 0}
    <div class="empty-state error">
      <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/>
        <line x1="12" y1="16" x2="12.01" y2="16"/>
      </svg>
      <p>Cannot connect to daemon</p>
      <p class="error-sub">Make sure labalaba-daemon is running on port 27015</p>
    </div>
  {:else if $tasks.length === 0}
    <div class="empty-state">
      <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1">
        <rect x="3" y="3" width="18" height="18" rx="2"/><path d="M9 12h6"/><path d="M12 9v6"/>
      </svg>
      <p>No tasks yet</p>
      <button class="btn btn-primary" onclick={onAddNew}>Add your first task</button>
    </div>
  {:else}
    <div class="cards">
      {#each $tasks as task (task.config.id['0'])}
        <TaskCard {task} {onViewLogs} {onEdit} />
      {/each}
    </div>
  {/if}
</section>

<style>
  .task-list {
    padding: 1.25rem;
    flex: 1;
    overflow-y: auto;
  }

  .list-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 1rem;
  }

  .section-title {
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }

  .cards {
    display: flex;
    flex-direction: column;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.75rem;
    padding: 4rem 1rem;
    color: var(--text-muted);
    text-align: center;
  }

  .empty-state.error { color: var(--status-crashed); }
  .error-sub { font-size: 0.75rem; color: var(--text-muted); }

  .spinner {
    width: 28px;
    height: 28px;
    border: 2px solid var(--border-subtle);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin { to { transform: rotate(360deg); } }
</style>
