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
  import { check, type Update } from '@tauri-apps/plugin-updater';
  import { relaunch } from '@tauri-apps/plugin-process';
  import { invoke } from '@tauri-apps/api/core';

  // Modal state
  let showForm = $state(false);
  let editingTask = $state<TaskDto | undefined>(undefined);
  let showSettings = $state(false);
  let activeLogTaskId = $state<string | null>(null);
  let updateInfo = $state<UpdateInfo | null>(null);

  // Self-update state. `selfUpdate` is null when this platform can't install in
  // place (.deb/.rpm, Intel Mac, or no signed manifest yet) — the modal then
  // falls back to linking at the release page.
  type InstallPhase = 'idle' | 'downloading' | 'installing' | 'error';
  let selfUpdate = $state<Update | null>(null);
  let phase = $state<InstallPhase>('idle');
  let downloaded = $state(0);
  let contentLength = $state(0);
  let installError = $state<string | null>(null);
  let daemonStopped = $state(false);

  const busy = $derived(phase === 'downloading' || phase === 'installing');
  const progressPct = $derived(
    contentLength > 0 ? Math.min(100, Math.round((downloaded / contentLength) * 100)) : 0
  );

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
        if (!selfUpdate) {
          await probeSelfUpdate();
        }
      }
    } catch (e) {
      console.error('Failed to fetch pending update:', e);
    }
  }

  // The daemon tells us *that* an update exists; the updater plugin tells us
  // whether we can install it ourselves. A failure here is expected on
  // platforms without a signed updater artifact and is not surfaced — the
  // modal just keeps the manual download link.
  async function probeSelfUpdate() {
    try {
      selfUpdate = await check();
    } catch (e) {
      console.warn('Self-update unavailable; falling back to manual download:', e);
      selfUpdate = null;
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
    if (busy) return;
    updateInfo = null;
    // Reset so a later reopen offers Install again rather than being stuck on
    // the fallback link from an earlier failure.
    phase = 'idle';
    installError = null;
  }

  async function installUpdate() {
    if (!selfUpdate || busy) return;

    installError = null;
    downloaded = 0;
    contentLength = 0;
    daemonStopped = false;
    phase = 'downloading';

    try {
      await selfUpdate.download((e) => {
        switch (e.event) {
          case 'Started':
            contentLength = e.data.contentLength ?? 0;
            break;
          case 'Progress':
            downloaded += e.data.chunkLength;
            break;
        }
      });

      // download() verifies the signature before resolving, so the update is
      // both present and trusted by this point. Only now is it safe to stop the
      // daemon: doing it earlier would kill the user's running tasks for an
      // update that might never have arrived.
      phase = 'installing';
      await invoke('prepare_for_update');
      daemonStopped = true;
      await selfUpdate.install();

      // Windows exits inside install() and never reaches this line; macOS and
      // Linux install in place and need the restart requested explicitly.
      await relaunch();
    } catch (e) {
      phase = 'error';
      installError = e instanceof Error ? e.message : String(e);

      // If we got as far as stopping the daemon, the user's tasks are down and
      // the app is talking to nothing. Failing the update is recoverable;
      // leaving them with a dead process manager is not.
      if (daemonStopped) {
        try {
          await invoke('start_daemon');
          daemonStopped = false;
          await loadTasks();
        } catch (restartErr) {
          installError = `${installError} — and the daemon could not be restarted: ${restartErr}. Restart Labalaba.`;
        }
      }
    }
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
        <button class="btn-icon" aria-label="Close" onclick={dismissUpdate} disabled={busy}>
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
        
        {#if selfUpdate && phase !== 'error'}
          <p class="update-warning">
            Installing stops every running task and restarts Labalaba.
          </p>
        {/if}

        {#if phase === 'downloading'}
          <div class="progress">
            <div
              class="progress-track"
              role="progressbar"
              aria-valuemin="0"
              aria-valuemax="100"
              aria-valuenow={contentLength > 0 ? progressPct : undefined}
              aria-label="Download progress"
            >
              <div class="progress-bar" style="width: {contentLength > 0 ? progressPct : 0}%"></div>
            </div>
            <span class="progress-label">
              {contentLength > 0 ? `Downloading… ${progressPct}%` : 'Downloading…'}
            </span>
          </div>
        {:else if phase === 'installing'}
          <p class="progress-label">Stopping tasks and installing…</p>
        {:else if phase === 'error'}
          <p class="update-error">Update failed: {installError}</p>
        {/if}

        <div class="update-footer">
          <button class="btn" onclick={dismissUpdate} disabled={busy}>
            Remind Me Later
          </button>
          {#if selfUpdate && phase !== 'error'}
            <button class="btn btn-primary" onclick={installUpdate} disabled={busy}>
              {busy ? 'Working…' : 'Install Update'}
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

  .update-error {
    font-size: 0.8125rem;
    color: var(--danger);
    margin: 0;
    word-break: break-word;
  }

  .progress {
    display: flex;
    flex-direction: column;
    gap: 0.375rem;
  }

  .progress-track {
    height: 6px;
    border-radius: 3px;
    background: var(--bg-surface);
    overflow: hidden;
  }

  .progress-bar {
    height: 100%;
    border-radius: 3px;
    background: var(--accent);
    transition: width 120ms linear;
  }

  .progress-label {
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
