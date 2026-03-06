<script lang="ts">
  import { getVersion } from '@tauri-apps/api/app';
  import { theme } from '$lib/stores/theme';
  import { stats } from '$lib/stores/tasks';

  let { onSettingsClick } = $props<{ onSettingsClick: () => void }>();

  let appVersion = $state('');
  getVersion().then(v => appVersion = v);

  function sunIcon() {
    return `<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <circle cx="12" cy="12" r="5"/><line x1="12" y1="1" x2="12" y2="3"/>
      <line x1="12" y1="21" x2="12" y2="23"/><line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/>
      <line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/><line x1="1" y1="12" x2="3" y2="12"/>
      <line x1="21" y1="12" x2="23" y2="12"/><line x1="4.22" y1="19.78" x2="5.64" y2="18.36"/>
      <line x1="18.36" y1="5.64" x2="19.78" y2="4.22"/>
    </svg>`;
  }

  function moonIcon() {
    return `<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/>
    </svg>`;
  }
</script>

<header class="topbar glass">
  <!-- Branding -->
  <div class="brand">
    <img src="/logo.jpg" alt="Labalaba" class="brand-logo" />
    <span class="brand-name">Labalaba <span class="brand-version">v{appVersion}</span></span>
  </div>

  <!-- Stats summary -->
  <div class="stats-row">
    <div class="stat">
      <span class="stat-value running">{$stats.running}</span>
      <span class="stat-label">Running</span>
    </div>
    <div class="divider"></div>
    <div class="stat">
      <span class="stat-value">{$stats.stopped}</span>
      <span class="stat-label">Stopped</span>
    </div>
    <div class="divider"></div>
    <div class="stat">
      <span class="stat-value crashed">{$stats.crashed}</span>
      <span class="stat-label">Crashed</span>
    </div>
    <div class="divider"></div>
    <div class="stat">
      <span class="stat-value">{$stats.total}</span>
      <span class="stat-label">Total</span>
    </div>
  </div>

  <!-- Actions -->
  <div class="actions">
    <button class="btn-icon" title="Toggle theme" onclick={() => theme.toggle()}>
      {@html $theme === 'dark' ? sunIcon() : moonIcon()}
    </button>
    <button class="btn-icon" title="Settings" onclick={onSettingsClick}>
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="3"/>
        <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06
          a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09
          A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83
          l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09
          A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83
          l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09
          a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83
          l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09
          a1.65 1.65 0 0 0-1.51 1z"/>
      </svg>
    </button>
  </div>
</header>

<style>
  .topbar {
    display: flex;
    align-items: center;
    gap: 1.5rem;
    padding: 0 1.25rem;
    height: 52px;
    border-radius: 0;
    border-left: none;
    border-right: none;
    border-top: none;
    position: sticky;
    top: 0;
    z-index: 50;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-shrink: 0;
  }

  .brand-logo {
    width: 28px;
    height: 28px;
    border-radius: 6px;
    object-fit: cover;
  }

  .brand-name {
    font-size: 1rem;
    font-weight: 700;
    letter-spacing: -0.02em;
    color: var(--text-primary);
  }

  .brand-version {
    font-size: 0.75rem;
    font-weight: 400;
    color: var(--text-muted);
    letter-spacing: 0;
  }

  .stats-row {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    flex: 1;
    justify-content: center;
  }

  .stat {
    display: flex;
    align-items: baseline;
    gap: 0.3rem;
  }

  .stat-value {
    font-size: 1.125rem;
    font-weight: 700;
    color: var(--text-primary);
    line-height: 1;
  }

  .stat-value.running { color: var(--status-running); }
  .stat-value.crashed { color: var(--status-crashed); }

  .stat-label {
    font-size: 0.6875rem;
    font-weight: 500;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }

  .divider {
    width: 1px;
    height: 18px;
    background: var(--border-subtle);
  }

  .actions {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    flex-shrink: 0;
  }
</style>
