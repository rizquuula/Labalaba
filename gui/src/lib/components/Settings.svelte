<script lang="ts">
  import { settings, loadSettings, saveSettings } from '$lib/stores/settings';
  import { api, type UpdateInfo } from '$lib/api/client';
  import { theme } from '$lib/stores/theme';
  import { onMount } from 'svelte';

  let { onClose } = $props<{ onClose: () => void }>();

  let draft = $state({ ...$settings });
  let saving = $state(false);
  let updateInfo = $state<UpdateInfo | null>(null);
  let checkingUpdate = $state(false);
  let updateError = $state<string | null>(null);

  onMount(() => loadSettings());

  // Keep draft in sync when settings load
  $effect(() => { draft = { ...$settings }; });

  async function handleSave() {
    saving = true;
    try {
      await saveSettings(draft);
      if (draft.theme !== $theme) {
        theme.set(draft.theme as 'dark' | 'light');
      }
      onClose();
    } finally {
      saving = false;
    }
  }

  async function checkForUpdates() {
    checkingUpdate = true;
    updateError = null;
    updateInfo = null;
    try {
      updateInfo = await api.update.check();
    } catch (e) {
      updateError = String(e);
    } finally {
      checkingUpdate = false;
    }
  }
</script>

<div class="modal-backdrop" role="dialog" aria-modal="true">
  <div class="modal glass-strong">
    <div class="modal-header">
      <h2>Settings</h2>
      <button class="btn-icon" aria-label="Close" onclick={onClose}>
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
        </svg>
      </button>
    </div>

    <div class="settings-sections">
      <!-- Appearance -->
      <section class="settings-section">
        <h3 class="section-heading">Appearance</h3>
        <div class="setting-row">
          <div>
            <p class="setting-name">Theme</p>
          </div>
          <select class="input select-sm" bind:value={draft.theme}>
            <option value="dark">Dark</option>
            <option value="light">Light</option>
          </select>
        </div>
      </section>

      <!-- Daemon -->
      <section class="settings-section">
        <h3 class="section-heading">Daemon</h3>
        <div class="setting-row">
          <div>
            <p class="setting-name">Daemon Port</p>
            <p class="setting-desc">Local port for HTTP API and WebSocket</p>
          </div>
          <input class="input input-sm" type="number" min="1024" max="65535"
            bind:value={draft.daemon_port} />
        </div>
        <div class="setting-row">
          <div>
            <p class="setting-name">Config File Path</p>
            <p class="setting-desc">Path to tasks.yaml</p>
          </div>
          <input class="input input-sm" type="text" bind:value={draft.config_path} />
        </div>
        <div class="setting-row">
          <div>
            <p class="setting-name">Log Buffer (lines)</p>
            <p class="setting-desc">Max log lines kept in memory per task</p>
          </div>
          <input class="input input-sm" type="number" min="100" max="50000"
            bind:value={draft.log_buffer_lines} />
        </div>
      </section>

      <!-- Notifications -->
      <section class="settings-section">
        <h3 class="section-heading">Notifications</h3>
        <div class="setting-row">
          <div>
            <p class="setting-name">Desktop Notifications</p>
            <p class="setting-desc">Notify when a task crashes or stops</p>
          </div>
          <label class="toggle">
            <input type="checkbox" bind:checked={draft.notifications_enabled} />
            <span class="toggle-track"></span>
          </label>
        </div>
        <div class="setting-row">
          <div>
            <p class="setting-name">Launch on Startup</p>
            <p class="setting-desc">Start Labalaba with Windows</p>
          </div>
          <label class="toggle">
            <input type="checkbox" bind:checked={draft.launch_on_startup} />
            <span class="toggle-track"></span>
          </label>
        </div>
      </section>

      <!-- Updates -->
      <section class="settings-section">
        <h3 class="section-heading">Updates</h3>
        <div class="setting-row">
          <div>
            <p class="setting-name">Auto-check for Updates</p>
          </div>
          <label class="toggle">
            <input type="checkbox" bind:checked={draft.auto_check_updates} />
            <span class="toggle-track"></span>
          </label>
        </div>

        <div class="update-check-area">
          <button class="btn" onclick={checkForUpdates} disabled={checkingUpdate}>
            {checkingUpdate ? 'Checking…' : 'Check for Updates Now'}
          </button>

          {#if updateInfo}
            {#if updateInfo.available}
              <div class="update-available">
                <p class="update-label">Update available: <strong>{updateInfo.latest_version}</strong></p>
                {#if updateInfo.release_url}
                  <a href={updateInfo.release_url} target="_blank" class="btn btn-primary">
                    View Release
                  </a>
                {/if}
              </div>
            {:else}
              <p class="update-current">You're on the latest version ({updateInfo.current_version})</p>
            {/if}
          {/if}

          {#if updateError}
            <p class="update-error">{updateError}</p>
          {/if}
        </div>
      </section>
    </div>

    <div class="modal-footer">
      <button type="button" class="btn" onclick={onClose}>Cancel</button>
      <button type="button" class="btn btn-primary" onclick={handleSave} disabled={saving}>
        {saving ? 'Saving…' : 'Save Settings'}
      </button>
    </div>
  </div>
</div>

<style>
  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 1.25rem;
  }

  .modal-header h2 {
    font-size: 1.0625rem;
    font-weight: 700;
  }

  .settings-sections {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .settings-section {
    margin-bottom: 0.75rem;
  }

  .section-heading {
    font-size: 0.6875rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--text-muted);
    margin-bottom: 0.5rem;
    padding-bottom: 0.375rem;
    border-bottom: 1px solid var(--border-subtle);
  }

  .setting-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    padding: 0.5rem 0;
  }

  .setting-name {
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--text-primary);
  }

  .setting-desc {
    font-size: 0.75rem;
    color: var(--text-muted);
    margin-top: 0.125rem;
  }

  .input-sm { width: 140px; }
  .select-sm { width: 100px; }

  .update-check-area {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    flex-wrap: wrap;
    padding: 0.5rem 0;
  }

  .update-available {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .update-label { font-size: 0.875rem; color: var(--status-running); }
  .update-current { font-size: 0.8125rem; color: var(--text-muted); }
  .update-error { font-size: 0.8125rem; color: var(--status-crashed); }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
    margin-top: 1.25rem;
    padding-top: 1rem;
    border-top: 1px solid var(--border-subtle);
  }
</style>
