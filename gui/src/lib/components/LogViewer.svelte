<script lang="ts">
  import { onMount, onDestroy, tick } from 'svelte';
  import { get } from 'svelte/store';
  import { connectLogStream, fetchHistoricalLogs, type LogEntry } from '$lib/api/websocket';
  import { settings } from '$lib/stores/settings';

  let { taskId, taskName, onClose } = $props<{
    taskId: string;
    taskName: string;
    onClose: () => void;
  }>();

  // Identical log lines (same timestamp/stream/text) legitimately recur, so we
  // tag every displayed entry with a unique key — keying the {#each} by content
  // would throw on duplicates once a lot of history is loaded.
  type KeyedLog = LogEntry & { _k: number };
  let seq = 0;
  function tag(entry: LogEntry): KeyedLog {
    return { ...entry, _k: seq++ };
  }

  let container: HTMLDivElement;
  let logs = $state<KeyedLog[]>([]);
  let autoScroll = $state(true);
  let loadingHistory = $state(false);
  let loadingOlder = $state(false);
  let hasMore = $state(false);
  let disconnect: (() => void) | null = null;
  let destroyed = false;

  // Lines fetched per request and kept in the live tail, driven by the
  // "Log Buffer (lines)" setting (clamped to its UI bounds). "Load older" can
  // grow the buffer past this so earlier history stays visible once pulled.
  const pageSize = Math.min(50000, Math.max(100, get(settings).log_buffer_lines || 5000));

  // Paging bookkeeping: `historyLoaded` counts non-live disk lines pulled so far
  // (the offset base for the next older page); `liveCount` counts lines appended
  // live since open; `olderCount` counts lines prepended via "load older" so the
  // live-tail cap never trims them away.
  let historyLoaded = 0;
  let liveCount = 0;
  let olderCount = 0;

  function liveCap(): number {
    return pageSize + olderCount;
  }

  async function appendLive(entry: LogEntry) {
    liveCount += 1;
    logs = [...logs.slice(-(liveCap() - 1)), tag(entry)];
    if (autoScroll) {
      await tick();
      container?.scrollTo({ top: container.scrollHeight });
    }
  }

  function sameEntry(a: LogEntry, b: LogEntry): boolean {
    return a.timestamp === b.timestamp && a.stream === b.stream && a.line === b.line;
  }

  /**
   * Length of the largest run where a suffix of `older` equals a prefix of
   * `newer` (lines double-counted across a page boundary). Returns 0 when
   * nothing overlaps. Bounded by the shorter of the two arrays.
   */
  function boundaryOverlap(older: LogEntry[], newer: LogEntry[]): number {
    const max = Math.min(older.length, newer.length);
    for (let k = max; k > 0; k--) {
      let matches = true;
      for (let i = 0; i < k; i++) {
        if (!sameEntry(older[older.length - k + i], newer[i])) {
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

    const history = await fetchHistoricalLogs(taskId, pageSize, 0);
    historyLoaded = history.length;
    // A full page of history implies there may be older lines to page into.
    hasMore = history.length >= pageSize;

    // Each backend line is BOTH written to the history file AND emitted live, so
    // a line produced inside the fetch window can appear in history *and* in the
    // buffer. Those overlapping lines are necessarily a contiguous run that ends
    // history and starts the buffer, so we only strip the largest boundary
    // overlap (buffer prefix == history suffix) rather than globally de-duping —
    // that preserves legitimately-repeated output elsewhere in the stream.
    const overlap = boundaryOverlap(history, buffer);
    logs = history.map(tag);
    const flushed = buffer.slice(overlap);
    for (const entry of flushed) {
      logs = [...logs.slice(-(liveCap() - 1)), tag(entry)];
    }
    liveCount += flushed.length;
    buffer.length = 0;
    liveReady = true;
    loadingHistory = false;

    if (autoScroll && logs.length > 0) {
      await tick();
      container?.scrollTo({ top: container.scrollHeight });
    }
  });

  async function loadOlder() {
    if (loadingOlder || !hasMore || !container) return;
    loadingOlder = true;
    try {
      // Skip everything currently newer than our oldest line: the disk history
      // already pulled plus the live lines appended since (also written to disk).
      const offset = historyLoaded + liveCount;
      const older = await fetchHistoricalLogs(taskId, pageSize, offset);

      // While the file grows live the page can reach slightly too far forward;
      // strip any lines that overlap what we already show.
      const overlap = boundaryOverlap(older, logs);
      const fresh = overlap > 0 ? older.slice(0, older.length - overlap) : older;

      if (fresh.length > 0) {
        const prevHeight = container.scrollHeight;
        const prevTop = container.scrollTop;
        logs = [...fresh.map(tag), ...logs];
        olderCount += fresh.length;
        historyLoaded += fresh.length;
        await tick();
        // Keep the viewport anchored on the same line after prepending.
        container.scrollTop = container.scrollHeight - prevHeight + prevTop;
      }

      // A short page (or one fully consumed by overlap) means we've hit the start.
      hasMore = older.length >= pageSize && fresh.length > 0;
    } finally {
      loadingOlder = false;
    }
  }

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
      {#if hasMore}
        <button class="load-older" onclick={loadOlder} disabled={loadingOlder}>
          {loadingOlder ? 'Loading older lines…' : 'Load older lines'}
        </button>
      {/if}
      {#each logs as entry (entry._k)}
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

  .load-older {
    display: block;
    width: 100%;
    margin: 0 0 0.375rem;
    padding: 0.3rem 0.5rem;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-secondary);
    font-size: 0.75rem;
    font-family: inherit;
    cursor: pointer;
  }

  .load-older:hover:not(:disabled) {
    color: var(--text-primary);
    border-color: var(--text-muted);
  }

  .load-older:disabled {
    opacity: 0.6;
    cursor: default;
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
