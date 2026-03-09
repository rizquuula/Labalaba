<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import TopBar from '$lib/components/TopBar.svelte';
  import TaskList from '$lib/components/TaskList.svelte';
  import TaskForm from '$lib/components/TaskForm.svelte';
  import LogViewer from '$lib/components/LogViewer.svelte';
  import Settings from '$lib/components/Settings.svelte';
  import { loadTasks, startPolling, tasks } from '$lib/stores/tasks';
  import type { TaskDto } from '$lib/api/client';
  import type { UpdateInfo } from '$lib/api/client';

  // Modal state
  let showForm = $state(false);
  let editingTask = $state<TaskDto | undefined>(undefined);
  let showSettings = $state(false);
  let activeLogTaskId = $state<string | null>(null);
  let updateInfo = $state<UpdateInfo | null>(null);

  const activeLogTask = $derived(
    activeLogTaskId ? $tasks.find(t => t.config.id === activeLogTaskId) : null
  );

  let stopPolling: (() => void) | null = null;

  onMount(async () => {
    await loadTasks();
    stopPolling = startPolling();
    
    // Listen for update available events from daemon
    const unlisten = await listen<UpdateInfo>('update-available', (event) => {
      updateInfo = event.payload;
    });
    
    // Cleanup listener on destroy
    onDestroy(() => {
      unlisten();
    });
  });

  onDestroy(() => stopPolling?.());

  function openAddForm() {
    editingTask = undefined;
    showForm = true;
  }

  function openEditForm(task: TaskDto) {
    editingTask = task;
    showForm = true;
  }

  function closeForm() {
    showForm = false;
    editingTask = undefined;
  }

  function openLogs(id: string) {
    activeLogTaskId = activeLogTaskId === id ? null : id;
  }

  function closeLogs() {
    activeLogTaskId = null;
  }

  function dismissUpdate() {
    updateInfo = null;
  }
</script>

<div class="layout">
  <TopBar onSettingsClick={() => (showSettings = true)} />

  <main class="main-content">
    <TaskList onViewLogs={openLogs} onEdit={openEditForm} onAddNew={openAddForm} />

    {#if activeLogTask}
      <LogViewer
        taskId={activeLogTask.config.id}
        taskName={activeLogTask.config.description}
        onClose={closeLogs}
      />
    {/if}
  </main>
</div>

{#if showForm}
  <TaskForm task={editingTask} onClose={closeForm} />
{/if}

{#if showSettings}
  <Settings onClose={() => (showSettings = false)} />
{/if}

{#if updateInfo}
  <div class="update-modal-backdrop" role="dialog" aria-modal="true">
    <div class="update-modal glass-strong">
      <div class="modal-header">
        <h2>New Version Available</h2>
        <button class="btn-icon" aria-label="Close" onclick={dismissUpdate}>
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </button>
      </div>
      
      <div class="update-content">
        <p class="update-version">
          Update available: <strong>{updateInfo.current_version}</strong> → <strong>{updateInfo.latest_version}</strong>
        </p>
        
        {#if updateInfo.release_notes}
          <div class="release-notes">
            <h3>Release Notes</h3>
            <p>{updateInfo.release_notes}</p>
          </div>
        {/if}
        
        {#if updateInfo.release_url}
          <a href={updateInfo.release_url} target="_blank" class="btn btn-primary">
            Download Update
          </a>
        {/if}
        
        <button class="btn" onclick={dismissUpdate}>
          Remind Me Later
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .layout {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
  }

  .main-content {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
  }

  .update-modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    padding: 1rem;
  }

  .update-modal {
    background: var(--bg-primary);
    border-radius: 0.75rem;
    padding: 1.5rem;
    max-width: 500px;
    width: 100%;
    box-shadow: 0 10px 40px rgba(0, 0, 0, 0.3);
  }

  .update-content {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    margin-top: 1rem;
  }

  .update-version {
    font-size: 0.9375rem;
    color: var(--text-primary);
    margin: 0;
  }

  .release-notes {
    background: var(--bg-secondary);
    border-radius: 0.5rem;
    padding: 1rem;
    max-height: 200px;
    overflow-y: auto;
  }

  .release-notes h3 {
    font-size: 0.8125rem;
    font-weight: 600;
    margin: 0 0 0.5rem 0;
    color: var(--text-muted);
  }

  .release-notes p {
    font-size: 0.8125rem;
    color: var(--text-primary);
    margin: 0;
    white-space: pre-wrap;
  }

  .update-content .btn {
    width: 100%;
  }
</style>
