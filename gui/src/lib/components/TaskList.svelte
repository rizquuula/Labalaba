<script lang="ts">
  import { tasks, tasksLoading, tasksError } from '$lib/stores/tasks';
  import { settings } from '$lib/stores/settings';
  import { derived } from 'svelte/store';
  import TaskCard from './TaskCard.svelte';
  import type { TaskDto } from '$lib/api/client';

  let { onViewLogs, onEdit, onAddNew } = $props<{
    onViewLogs: (id: string) => void;
    onEdit: (task: TaskDto) => void;
    onAddNew: () => void;
  }>();

  let searchQuery = $state('');
  let statusFilter = $state<'all' | 'running' | 'stopped' | 'crashed'>('all');

  const filteredTasks = derived(tasks, ($tasks) => {
    return $tasks.filter(task => {
      const query = searchQuery.toLowerCase();
      const matchesSearch = !searchQuery || 
        task.config.description.toLowerCase().includes(query) ||
        task.config.executable.toLowerCase().includes(query);
      
      const matchesStatus = statusFilter === 'all' || task.status === statusFilter;
      
      return matchesSearch && matchesStatus;
    });
  });
</script>

<section class="task-list">
  <div class="list-header">
    <h2 class="section-title">Tasks</h2>
    <button class="btn btn-primary" onclick={onAddNew}>
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
        <line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>
      </svg>
      New Task
    </button>
  </div>

  {#if $tasksLoading && $tasks.length === 0}
    <div class="empty-state" role="status" aria-label="Connecting to daemon">
      <div class="spinner"></div>
      <p>Connecting to daemon…</p>
    </div>
  {:else if $tasksError && $tasks.length === 0}
    <div class="empty-state error">
      <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" aria-hidden="true">
        <circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/>
        <line x1="12" y1="16" x2="12.01" y2="16"/>
      </svg>
      <p>Cannot connect to daemon</p>
      <p class="error-sub">Make sure the daemon is running on the configured port ({$settings.daemon_port})</p>
    </div>
  {:else if $tasks.length === 0}
    <div class="empty-state">
      <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" aria-hidden="true">
        <rect x="3" y="3" width="18" height="18" rx="2"/><path d="M9 12h6"/><path d="M12 9v6"/>
      </svg>
      <p>No tasks yet</p>
      <button class="btn btn-primary" onclick={onAddNew}>Add your first task</button>
    </div>
  {:else}
    {#if $tasksError}
      <div class="connection-banner" role="alert">
        Connection lost — showing last known state
      </div>
    {/if}

    <div class="filters">
      <div class="search-input">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
          <circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/>
        </svg>
        <input
          type="text"
          placeholder="Search tasks…"
          aria-label="Search tasks"
          bind:value={searchQuery}
          class="input input-sm"
        />
        {#if searchQuery}
          <button class="btn-icon" onclick={() => searchQuery = ''} title="Clear">
            <svg width="14" height="14" viewBox="0 0 24 24"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
          </button>
        {/if}
      </div>
      <select class="input filter-select" bind:value={statusFilter}>
        <option value="all">All statuses</option>
        <option value="running">Running</option>
        <option value="stopped">Stopped</option>
        <option value="crashed">Crashed</option>
      </select>
    </div>

    <div class="cards">
      {#each $filteredTasks as task (task.config.id)}
        <TaskCard {task} {onViewLogs} {onEdit} />
      {/each}
      {#if $filteredTasks.length === 0}
        <div class="empty-state">
          <p>No tasks match your search</p>
        </div>
      {/if}
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

  .filters {
    display: flex;
    gap: 0.75rem;
    margin-bottom: 1rem;
    align-items: center;
  }

  .search-input {
    flex: 1;
    position: relative;
    display: flex;
    align-items: center;
  }

  .search-input svg {
    position: absolute;
    left: 0.75rem;
    color: var(--text-muted);
    pointer-events: none;
  }

  .search-input input {
    padding-left: 2.25rem;
    padding-right: 2rem;
    width: 100%;
  }

  .search-input .btn-icon {
    position: absolute;
    right: 0.5rem;
    color: var(--text-muted);
  }

  .filter-select {
    width: 140px;
  }

  .connection-banner {
    margin-bottom: 0.75rem;
    padding: 0.5rem 0.75rem;
    border-radius: var(--radius-sm);
    background: color-mix(in srgb, var(--status-crashed) 12%, transparent);
    border: 1px solid color-mix(in srgb, var(--status-crashed) 30%, transparent);
    color: var(--status-crashed);
    font-size: 0.8125rem;
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
