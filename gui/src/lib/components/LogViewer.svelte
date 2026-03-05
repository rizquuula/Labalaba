<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { connectLogStream, fetchHistoricalLogs, type LogEntry } from '$lib/api/websocket';

  let { taskId, taskName, onClose } = $props<{
    taskId: string;
    taskName: string;
    onClose: () => void;
  }>();

  let container: HTMLDivElement;
  let logs = $state<LogEntry[]>([]);
  let autoScroll = $state(true);
  let loadingHistory = $state(false);
  let disconnect: (() => void) | null = null;

  onMount(async () => {
    loadingHistory = true;
    
    const history = await fetchHistoricalLogs(taskId, 500);
    logs = history;
    loadingHistory = false;
    
    if (autoScroll && logs.length > 0) {
      setTimeout(() => {
        container?.scrollTo({ top: container.scrollHeight });
      }, 0);
    }
    
    disconnect = connectLogStream(taskId, (entry) => {
      logs = [...logs.slice(-4999), entry];
      if (autoScroll) {
        setTimeout(() => {
          container?.scrollTo({ top: container.scrollHeight });
        }, 0);
      }
    });
  });

  onDestroy(() => disconnect?.());

  function clearLogs() { logs = []; }

  function getLineClass(entry: LogEntry) {
    return entry.stream === 'stderr' ? 'line-err' : 'line-out';
  }
</script>

<div class="log-panel glass-strong">
  <div class="log-header">
    <div class="log-title">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <polyline points="4 17 10 11 4 5"/><line x1="12" y1="19" x2="20" y2="19"/>
      </svg>
      <span>{taskName}</span>
    </div>
    <div class="log-actions">
      <label class="autoscroll-toggle">
        <input type="checkbox" bind:checked={autoScroll} />
        <span>Auto-scroll</span>
      </label>
      <button class="btn-icon" title="Clear" onclick={clearLogs}>
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="3 6 5 6 21 6"/>
          <path d="M19 6l-1 14H6L5 6"/>
          <path d="M9 6V4h6v2"/>
        </svg>
      </button>
      <button class="btn-icon" title="Close" onclick={onClose}>
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
        </svg>
      </button>
    </div>
  </div>

  <div class="log-body" bind:this={container}>
    {#if loadingHistory}
      <p class="log-loading">Loading historical logs…</p>
    {:else if logs.length === 0}
      <p class="log-empty">Waiting for output…</p>
    {:else}
      {#each logs as entry (entry.timestamp + entry.line)}
        <div class={`log-line ${getLineClass(entry)}`}>
          <span class="log-ts">{entry.timestamp.slice(11, 19)}</span>
          <span class="log-text">{entry.line}</span>
        </div>
      {/each}
    {/if}
  </div>
</div>

<style>
  .log-panel {
    display: flex;
    flex-direction: column;
    border-radius: var(--radius-md);
    overflow: hidden;
    height: 320px;
    margin: 0 1.25rem 1.25rem;
  }

  .log-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.5rem 0.75rem;
    border-bottom: 1px solid var(--border-subtle);
    flex-shrink: 0;
  }

  .log-title {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    font-size: 0.8125rem;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .log-actions {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .autoscroll-toggle {
    display: flex;
    align-items: center;
    gap: 0.3rem;
    font-size: 0.75rem;
    color: var(--text-muted);
    cursor: pointer;
  }

  .autoscroll-toggle input { cursor: pointer; }

  .log-body {
    flex: 1;
    overflow-y: auto;
    padding: 0.5rem;
    font-family: 'Cascadia Code', 'Fira Code', 'Consolas', monospace;
    font-size: 0.75rem;
    line-height: 1.6;
  }

  .log-line {
    display: flex;
    gap: 0.625rem;
    white-space: pre-wrap;
    word-break: break-all;
  }

  .log-ts {
    flex-shrink: 0;
    color: var(--text-muted);
    user-select: none;
  }

  .line-out .log-text { color: var(--text-primary); }
  .line-err .log-text { color: var(--status-crashed); }

  .log-empty {
    padding: 1rem;
    color: var(--text-muted);
    font-size: 0.8125rem;
    font-style: italic;
  }

  .log-loading {
    padding: 1rem;
    color: var(--text-secondary);
    font-size: 0.8125rem;
    font-style: italic;
  }
</style>
