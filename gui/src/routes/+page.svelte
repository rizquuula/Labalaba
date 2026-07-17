<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import TopBar from '$lib/components/TopBar.svelte';
  import TaskList from '$lib/components/TaskList.svelte';
  import TaskForm from '$lib/components/TaskForm.svelte';
  import LogViewer from '$lib/components/LogViewer.svelte';
  import Settings from '$lib/components/Settings.svelte';
  import DaemonStatusBar from '$lib/components/DaemonStatusBar.svelte';
  import { loadTasks, startPolling, tasks } from '$lib/stores/tasks';
  import { api } from '$lib/api/client';
  import type { TaskDto } from '$lib/api/client';
  import type { UpdateInfo } from '$lib/api/client';
  import { focusTrap } from '$lib/actions/focusTrap';
  import { portal } from '$lib/actions/portal';
  import UpdateProgress from '$lib/components/UpdateProgress.svelte';
  import {
    selfUpdate,
    installPhase,
    installBusy,
    probeSelfUpdate,
    installSelfUpdate,
    resetInstallState
  } from '$lib/stores/selfUpdate';

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
  let updatePollInterval: ReturnType<typeof setInterval> | null = null;

  async function checkPendingUpdate() {
    try {
      const pending = await api.update.pending();
      if (pending && pending.available) {
        updateInfo = pending;
        if (!$selfUpdate) {
          await probeSelfUpdate();
        }
      }
    } catch (e) {
      console.error('Failed to fetch pending update:', e);
    }
  }

  onMount(async () => {
    await loadTasks();
    stopPolling = startPolling();

    await checkPendingUpdate();
    updatePollInterval = setInterval(checkPendingUpdate, 60 * 60 * 1000);
  });

  onDestroy(() => {
    stopPolling?.();
    if (updatePollInterval !== null) {
      clearInterval(updatePollInterval);
    }
  });

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
    if ($installBusy) return;
    updateInfo = null;
    // Reset so a later reopen offers Install again rather than being stuck on
    // the fallback link from an earlier failure.
    resetInstallState();
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

  <DaemonStatusBar />
</div>

{#if showForm}
  <TaskForm task={editingTask} onClose={closeForm} />
{/if}

{#if showSettings}
  <Settings onClose={() => (showSettings = false)} />
{/if}

{#if updateInfo}
  <div class="update-modal-backdrop" role="dialog" aria-modal="true" aria-labelledby="update-modal-title" use:focusTrap={{ onClose: dismissUpdate }} use:portal>
    <div class="update-modal glass-strong">
      <div class="modal-header">
        <h2 id="update-modal-title">New Version Available</h2>
        <button class="btn-icon" aria-label="Close" onclick={dismissUpdate} disabled={$installBusy}>
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
        
        {#if $selfUpdate && $installPhase !== 'error'}
          <p class="update-warning">
            Installing stops every running task and restarts Labalaba.
          </p>
        {/if}

        <UpdateProgress />

        <div class="update-footer">
          <button class="btn" onclick={dismissUpdate} disabled={$installBusy}>
            Remind Me Later
          </button>
          {#if $selfUpdate && $installPhase !== 'error'}
            <button class="btn btn-primary" onclick={installSelfUpdate} disabled={$installBusy}>
              {$installBusy ? 'Working…' : 'Install Update'}
            </button>
          {:else if updateInfo.release_url}
            <a href={updateInfo.release_url} target="_blank" class="btn btn-primary">
              Download Update
            </a>
          {/if}
        </div>
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
    background: rgba(0 0 0 / 0.6);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    padding: 1rem;
  }

  .update-modal {
    background: var(--bg-glass-strong);
    border-radius: var(--radius-lg);
    padding: 1.5rem;
    max-width: 500px;
    width: 100%;
    max-height: 88vh;
    overflow-y: auto;
    box-shadow: var(--shadow-glass);
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
    background: var(--bg-surface);
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

  .update-warning {
    font-size: 0.8125rem;
    color: var(--text-muted);
    margin: 0;
  }

  .update-footer {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
    margin-top: 0.25rem;
    border-top: 1px solid var(--border-subtle);
    padding-top: 1rem;
  }
</style>
