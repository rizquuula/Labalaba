<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { api } from '$lib/api/client';

  interface DaemonStatus {
    running: boolean;
    port: number;
    autostart: boolean;
  }

  let status = $state<DaemonStatus | null>(null);
  let busy = $state(false);
  let actionLabel = $state('');
  let pollInterval: ReturnType<typeof setInterval> | null = null;

  async function refreshStatus() {
    try {
      status = await invoke<DaemonStatus>('daemon_status');
    } catch (e) {
      console.error('Failed to fetch daemon status:', e);
    }
  }

  onMount(() => {
    refreshStatus();
    pollInterval = setInterval(refreshStatus, 3000);
  });

  onDestroy(() => {
    if (pollInterval !== null) {
      clearInterval(pollInterval);
    }
  });

  async function handleStart() {
    busy = true;
    actionLabel = 'Starting…';
    try {
      await invoke('start_daemon');
      await refreshStatus();
    } catch (e) {
      console.error('Failed to start daemon:', e);
    } finally {
      busy = false;
      actionLabel = '';
    }
  }

  async function handleStop() {
    busy = true;
    actionLabel = 'Stopping…';
    try {
      await api.daemon.shutdown();
    } catch (_e) {
      // The daemon may drop the connection as it exits — treat that as success
    }
    await refreshStatus();
    busy = false;
    actionLabel = '';
  }

  async function handleRestart() {
    busy = true;
    actionLabel = 'Restarting…';
    try {
      try {
        await api.daemon.shutdown();
      } catch (_e) {
        // ignore — daemon may close the connection on shutdown
      }

      // Poll until stopped (max 20 × 200ms = 4s)
      for (let i = 0; i < 20; i++) {
        await new Promise<void>((r) => setTimeout(r, 200));
        await refreshStatus();
        if (!status?.running) break;
      }

      await invoke('start_daemon');

      // Poll until running (max 30 × 200ms = 6s)
      for (let i = 0; i < 30; i++) {
        await new Promise<void>((r) => setTimeout(r, 200));
        await refreshStatus();
        if (status?.running) break;
      }
    } catch (e) {
      console.error('Restart failed:', e);
    } finally {
      busy = false;
      actionLabel = '';
    }
  }
</script>

<footer class="daemon-bar glass">
  <div class="status-section">
    <span
      class="status-dot"
      class:running={status?.running}
      aria-hidden="true"
    ></span>
    <span class="status-label">
      {#if busy && actionLabel}
        {actionLabel}
      {:else if status?.running}
        Daemon running
      {:else}
        Daemon stopped
      {/if}
    </span>
    {#if status}
      <span class="port-badge">· :{status.port}</span>
    {/if}
    {#if status?.autostart}
      <span class="autostart-badge">autostart</span>
    {/if}
  </div>

  <div class="controls">
    {#if !status?.running && !busy}
      <button class="btn btn-sm btn-primary" onclick={handleStart} disabled={busy}>
        Start
      </button>
    {/if}
    {#if status?.running && !busy}
      <button class="btn btn-sm" onclick={handleRestart} disabled={busy}>
        Restart
      </button>
      <button class="btn btn-sm btn-danger" onclick={handleStop} disabled={busy}>
        Stop
      </button>
    {/if}
    {#if busy}
      <span class="busy-indicator" aria-live="polite">{actionLabel}</span>
    {/if}
  </div>
</footer>

<style>
  .daemon-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 1.25rem;
    height: 36px;
    border-radius: 0;
    border-left: none;
    border-right: none;
    border-bottom: none;
    flex-shrink: 0;
    z-index: 50;
  }

  .status-section {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    min-width: 0;
  }

  .status-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--status-crashed);
    flex-shrink: 0;
    transition: background 0.2s;
  }

  .status-dot.running {
    background: var(--status-running);
  }

  .status-label {
    font-size: 0.75rem;
    font-weight: 500;
    color: var(--text-secondary);
    white-space: nowrap;
  }

  .port-badge {
    font-size: 0.6875rem;
    color: var(--text-muted);
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }

  .autostart-badge {
    font-size: 0.625rem;
    font-weight: 600;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    color: var(--text-muted);
    background: var(--bg-surface-hover);
    border: 1px solid var(--border-subtle);
    border-radius: 4px;
    padding: 1px 5px;
    white-space: nowrap;
  }

  .controls {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    flex-shrink: 0;
  }

  .btn-sm {
    padding: 0.2rem 0.625rem;
    font-size: 0.75rem;
    line-height: 1.4;
  }

  .busy-indicator {
    font-size: 0.75rem;
    color: var(--text-muted);
    white-space: nowrap;
  }
</style>
