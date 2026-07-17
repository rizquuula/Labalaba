<script lang="ts">
  import { installPhase, contentLength, downloadPct, installError } from '$lib/stores/selfUpdate';
</script>

{#if $installPhase === 'downloading'}
  <div class="progress">
    <div
      class="progress-track"
      role="progressbar"
      aria-valuemin="0"
      aria-valuemax="100"
      aria-valuenow={$contentLength > 0 ? $downloadPct : undefined}
      aria-label="Download progress"
    >
      <div class="progress-bar" style="width: {$contentLength > 0 ? $downloadPct : 0}%"></div>
    </div>
    <span class="progress-label">
      {$contentLength > 0 ? `Downloading… ${$downloadPct}%` : 'Downloading…'}
    </span>
  </div>
{:else if $installPhase === 'installing'}
  <p class="progress-label">Stopping tasks and installing…</p>
{:else if $installPhase === 'error'}
  <p class="update-error">Update failed: {$installError}</p>
{/if}

<style>
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

  .update-error {
    font-size: 0.8125rem;
    color: var(--danger);
    margin: 0;
    word-break: break-word;
  }
</style>
