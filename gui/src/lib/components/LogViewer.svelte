<script lang="ts">
  import { onMount, onDestroy, tick } from 'svelte';
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
  let destroyed = false;

  async function appendLive(entry: LogEntry) {
    logs = [...logs.slice(-4999), entry];
    if (autoScroll) {
      await tick();
      container?.scrollTo({ top: container.scrollHeight });
    }
  }

  function sameEntry(a: LogEntry, b: LogEntry): boolean {
    return a.timestamp === b.timestamp && a.stream === b.stream && a.line === b.line;
  }

  /**
   * Length of the largest run where a prefix of `buffer` equals a suffix of
   * `history` (the lines double-counted across the fetch boundary). Returns 0
   * when nothing overlaps. Bounded by buffer length and history length.
   */
  function boundaryOverlap(history: LogEntry[], buffer: LogEntry[]): number {
    const max = Math.min(history.length, buffer.length);
    for (let k = max; k > 0; k--) {
      let matches = true;
      for (let i = 0; i < k; i++) {
        if (!sameEntry(history[history.length - k + i], buffer[i])) {
          matches = false;
          break;
        }
      }
      if (matches) return k;
    }
    return 0;
  }

  onMount(async () => {
    loadingHistory = true;

    // Buffer any live lines that arrive while history is loading; they are
    // appended after the snapshot so nothing emitted during the fetch is lost.
    let liveReady = false;
    const buffer: LogEntry[] = [];

    // Register the listener BEFORE fetching history so no lines slip through the
    // gap. await guarantees the listener is attached before we proceed.
    const stop = await connectLogStream(taskId, (entry) => {
      if (liveReady) {
        appendLive(entry);
      } else {
        buffer.push(entry);
      }
    });

    // If the viewer was destroyed while the listener was attaching, clean up now.
    if (destroyed) {
      stop();
      return;
    }
    disconnect = stop;

    const history = await fetchHistoricalLogs(taskId, 500);

    // Each backend line is BOTH written to the history file AND emitted live, so
    // a line produced inside the fetch window can appear in history *and* in the
    // buffer. Those overlapping lines are necessarily a contiguous run that ends
    // history and starts the buffer, so we only strip the largest boundary
    // overlap (buffer prefix == history suffix) rather than globally de-duping —
    // that preserves legitimately-repeated output elsewhere in the stream.
    const overlap = boundaryOverlap(history, buffer);
    logs = history;
    for (const entry of buffer.slice(overlap)) {
      logs = [...logs.slice(-4999), entry];
    }
    buffer.length = 0;
    liveReady = true;
    loadingHistory = false;

    if (autoScroll && logs.length > 0) {
      await tick();
      container?.scrollTo({ top: container.scrollHeight });
    }
  });

  onDestroy(() => {
    destroyed = true;
    disconnect?.();
  });

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
      <button class="btn-icon" title="Clear" aria-label="Clear logs" onclick={clearLogs}>
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true" focusable="false">
          <polyline points="3 6 5 6 21 6"/>
          <path d="M19 6l-1 14H6L5 6"/>
          <path d="M9 6V4h6v2"/>
        </svg>
      </button>
      <button class="btn-icon" title="Close" aria-label="Close logs" onclick={onClose}>
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true" focusable="false">
          <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
        </svg>
      </button>
    </div>
  </div>

  <div class="log-body" bind:this={container} aria-live={loadingHistory ? 'off' : 'polite'} aria-relevant="additions">
    {#if loadingHistory}
      <p class="log-loading">Loading historical logs…</p>
    {:else if logs.length === 0}
      <p class="log-empty">Waiting for output…</p>
    {:else}
      {#each logs as entry (entry.timestamp + entry.stream + entry.line)}
        <div class={`log-line ${getLineClass(entry)}`}>
          <span class="log-ts">{entry.timestamp.slice(11, 19)}</span>
          {#if entry.stream === 'stderr'}
            <span class="stream-tag" aria-hidden="true">ERR</span>
          {/if}
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
    height: clamp(180px, 30vh, 380px);
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

  .stream-tag {
    flex-shrink: 0;
    font-size: 0.6875rem;
    font-weight: 700;
    color: var(--status-crashed);
    letter-spacing: 0.04em;
    user-select: none;
  }

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
