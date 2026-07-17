<script lang="ts">
  import { untrack } from 'svelte';
  import { settings, loadSettings, saveSettings } from '$lib/stores/settings';
  import { api, resetConnection, type UpdateInfo } from '$lib/api/client';
  import { theme } from '$lib/stores/theme';
  import { focusTrap } from '$lib/actions/focusTrap';
  import { portal } from '$lib/actions/portal';
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { revealItemInDir } from '@tauri-apps/plugin-opener';
  import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';
  import UpdateProgress from '$lib/components/UpdateProgress.svelte';
  import {
    selfUpdate,
    installPhase,
    installBusy,
    probeSelfUpdate,
    installSelfUpdate
  } from '$lib/stores/selfUpdate';

  let { onClose } = $props<{ onClose: () => void }>();

  let draft = $state(untrack(() => ({ ...$settings })));
  let saving = $state(false);
  let updateInfo = $state<UpdateInfo | null>(null);
  let checkingUpdate = $state(false);
  let updateError = $state<string | null>(null);
  let saveError = $state<string | null>(null);

  // Danger Zone state
  let purgeData = $state(false);
  let showCleanupConfirm = $state(false);
  let cleaning = $state(false);
  let cleanupDone = $state<string | null>(null);
  let cleanupError = $state<string | null>(null);

  // Data Location state. Mirrors DataLocation in commands/portable.rs.
  interface DataLocation {
    data_dir: string;
    reveal_path: string;
    portable_active: boolean;
    portable_supported: boolean;
    portable_available: boolean;
    reason: string | null;
    target_dir: string | null;
    target_has_data: boolean;
    target_modified_ms: number | null;
  }

  let dataLocation = $state<DataLocation | null>(null);
  let switchingLocation = $state(false);
  let showPortableConfirm = $state(false);
  let locationNote = $state<string | null>(null);
  let locationError = $state<string | null>(null);

  const modalHeadingId = 'settings-heading';

  onMount(() => {
    void seedDraft();
    void loadDataLocation();
  });

  // Seed draft from freshly-loaded settings, using untrack so we don't re-run
  // when settings updates later. Also runs after a data-location switch, since
  // the daemon the draft came from has been replaced by then.
  async function seedDraft() {
    await loadSettings();
    untrack(() => { draft = { ...$settings }; });
    // Sync launch_on_startup with the real OS autostart state.
    try {
      draft.launch_on_startup = await invoke<boolean>('get_autostart');
    } catch {
      // Best-effort; leave the persisted value in place on error.
    }
  }

  async function loadDataLocation() {
    try {
      dataLocation = await invoke<DataLocation>('get_data_location');
    } catch (e) {
      locationError = String(e);
    }
  }

  async function revealData() {
    if (!dataLocation) return;
    try {
      await revealItemInDir(dataLocation.reveal_path);
    } catch (e) {
      locationError = String(e);
    }
  }

  // Open the confirm instead of flipping. The switch keeps showing the real
  // state until the move actually succeeds — it is not a preference, it moves
  // files and restarts the daemon.
  function requestPortableToggle(e: Event) {
    (e.currentTarget as HTMLInputElement).checked = dataLocation?.portable_active ?? false;
    locationNote = null;
    locationError = null;
    showPortableConfirm = true;
  }

  function portableConfirmMessage(loc: DataLocation): string {
    const target = loc.target_dir ?? '';
    const parts: string[] = [
      loc.portable_active
        ? `Labalaba will keep its data in ${target} again.`
        : `Labalaba will keep its data in ${target}, next to the app.`
    ];

    if (loc.target_has_data) {
      // The copy never overwrites, so whatever is already there is what loads.
      // Saying so is the whole point — otherwise a second toggle silently
      // resurrects an old task list.
      const when = loc.target_modified_ms
        ? new Date(loc.target_modified_ms).toLocaleString()
        : 'an unknown date';
      parts.push(
        `That folder already has a task file (last changed ${when}), and that is what will load. ` +
          `Your current tasks in ${loc.data_dir} are kept, but will not be used.`
      );
    } else {
      parts.push(
        `Your tasks, settings and logs are copied there. The originals in ${loc.data_dir} are kept as a backup.`
      );
    }

    if (!loc.portable_active) {
      parts.push(
        'Anyone who can write to that folder will be able to read the daemon access token, ' +
          'so prefer this only on a machine you do not share.'
      );
    }

    parts.push('The daemon restarts. Running tasks keep running.');
    return parts.join('\n\n');
  }

  async function applyPortableToggle() {
    if (!dataLocation) return;
    const next = !dataLocation.portable_active;
    switchingLocation = true;
    locationNote = null;
    locationError = null;
    try {
      dataLocation = await invoke<DataLocation>('set_portable_mode', { enabled: next });
      locationNote = `Data is now in ${dataLocation.data_dir}.`;
    } catch (e) {
      locationError = String(e);
      await loadDataLocation();
    } finally {
      showPortableConfirm = false;
      // In `finally`, not on success: the rollback path also restarts a daemon,
      // so the cached token is stale either way.
      resetConnection();
      await seedDraft();
      switchingLocation = false;
    }
  }

  // Coerce a possibly-empty/NaN number input to an integer within [min, max],
  // falling back to `fallback` when the value is missing/unparseable. HTML
  // min/max are advisory only — they don't clamp the bound value — so we clamp
  // here and ensure a real number (not a string) reaches the serde-typed Rust
  // fields.
  function clampInt(value: unknown, min: number, max: number, fallback: number): number {
    const n = typeof value === 'number' ? value : parseInt(String(value ?? ''), 10);
    if (!Number.isFinite(n)) return fallback;
    return Math.min(max, Math.max(min, Math.trunc(n)));
  }

  function sanitizeDraft() {
    draft.daemon_port = clampInt(draft.daemon_port, 1024, 65535, 27015);
    draft.log_buffer_lines = clampInt(draft.log_buffer_lines, 100, 50000, 5000);
    draft.log_max_file_size_mb = clampInt(draft.log_max_file_size_mb, 1, 1024, 10);
    draft.log_max_rotated_files = clampInt(draft.log_max_rotated_files, 0, 100, 5);
  }

  async function handleSave() {
    saving = true;
    saveError = null;
    try {
      sanitizeDraft();
      await saveSettings(draft);
      if (draft.theme !== $theme) {
        theme.set(draft.theme as 'dark' | 'light');
      }
      try {
        await invoke('set_autostart', { enabled: draft.launch_on_startup });
      } catch (e) {
        saveError = `Settings saved, but autostart update failed: ${String(e)}`;
        saving = false;
        return;
      }
      onClose();
    } catch (e) {
      saveError = String(e);
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
      // The daemon's answer only says an update exists. Ask the plugin the
      // separate question of whether we can install it here, or this panel
      // would offer nothing but a link to go do it by hand.
      if (updateInfo?.available) {
        await probeSelfUpdate();
      }
    } catch (e) {
      updateError = String(e);
    } finally {
      checkingUpdate = false;
    }
  }

  async function runCleanup() {
    cleaning = true;
    cleanupError = null;
    try {
      await invoke('cleanup_daemon', { purge: purgeData });
      showCleanupConfirm = false;
      cleanupDone = purgeData ? 'All data deleted.' : 'Background service removed.';
    } catch (e) {
      cleanupError = String(e);
    } finally {
      cleaning = false;
    }
  }
</script>

<div class="modal-backdrop" role="dialog" aria-modal="true" aria-labelledby={modalHeadingId} use:focusTrap={{ onClose }} use:portal>
  <div class="modal glass-strong">
    <div class="modal-header">
      <h2 id={modalHeadingId}>Settings</h2>
      <button class="btn-icon" aria-label="Close" onclick={onClose} disabled={saving}>
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
          <div class="setting-label-group">
            <p class="setting-name" id="label-theme">Theme</p>
          </div>
          <select class="input select-sm" id="setting-theme" aria-labelledby="label-theme" bind:value={draft.theme}>
            <option value="dark">Dark</option>
            <option value="light">Light</option>
          </select>
        </div>
      </section>

      <!-- Daemon -->
      <section class="settings-section">
        <h3 class="section-heading">Daemon</h3>
        <div class="setting-row">
          <div class="setting-label-group">
            <p class="setting-name" id="label-daemon-port">Daemon Port</p>
            <p class="setting-desc">Port used by the background daemon</p>
          </div>
          <input class="input input-sm" id="setting-daemon-port" type="number" min="1024" max="65535"
            aria-labelledby="label-daemon-port"
            bind:value={draft.daemon_port} />
        </div>
        <div class="setting-row">
          <div class="setting-label-group">
            <p class="setting-name" id="label-config-path">Config File Path</p>
            <p class="setting-desc">Path to tasks.yaml</p>
          </div>
          <input class="input input-sm" id="setting-config-path" type="text"
            aria-labelledby="label-config-path"
            bind:value={draft.config_path} />
        </div>
        <div class="setting-row">
          <div class="setting-label-group">
            <p class="setting-name" id="label-log-buffer">Log Buffer (lines)</p>
            <p class="setting-desc">Lines loaded when opening a log and kept in memory per task ("Load older" pages further back)</p>
          </div>
          <input class="input input-sm" id="setting-log-buffer" type="number" min="100" max="50000"
            aria-labelledby="label-log-buffer"
            bind:value={draft.log_buffer_lines} />
        </div>
      </section>

      <!-- Data Location -->
      <section class="settings-section">
        <h3 class="section-heading">Data Location</h3>

        {#if dataLocation}
          <div class="setting-row">
            <div class="setting-label-group">
              <p class="setting-name" id="label-data-dir">Data folder</p>
              <p class="setting-path" title={dataLocation.data_dir}>{dataLocation.data_dir}</p>
              <p class="setting-desc">tasks.yaml, settings.yaml and logs live here</p>
            </div>
            <button class="btn" onclick={revealData} disabled={switchingLocation}>Reveal</button>
          </div>

          {#if dataLocation.portable_supported}
            <div class="setting-row">
              <div class="setting-label-group">
                <p class="setting-name" id="label-portable">Keep data next to the app</p>
                <p class="setting-desc">
                  {#if !dataLocation.portable_available}
                    {dataLocation.reason}
                  {:else if dataLocation.portable_active}
                    Data is stored in the install folder. Turn this off to use your user profile
                    ({dataLocation.target_dir}).
                  {:else}
                    Store everything in {dataLocation.target_dir} instead of your user profile, so
                    the app and its data stay together.
                  {/if}
                </p>
              </div>
              <label class="toggle" aria-labelledby="label-portable">
                <input
                  type="checkbox"
                  checked={dataLocation.portable_active}
                  disabled={!dataLocation.portable_available || switchingLocation || $installBusy}
                  onchange={requestPortableToggle}
                />
                <span class="toggle-track"></span>
              </label>
            </div>
          {/if}

          {#if switchingLocation}
            <p class="setting-desc">Moving data and restarting the daemon…</p>
          {/if}
          {#if locationNote}
            <p class="cleanup-done">{locationNote}</p>
          {/if}
          {#if locationError}
            <p class="cleanup-error">{locationError}</p>
          {/if}
        {:else}
          <p class="setting-desc">Loading…</p>
        {/if}
      </section>

      <!-- Logs -->
      <section class="settings-section">
        <h3 class="section-heading">Logs</h3>
        <div class="setting-row">
          <div class="setting-label-group">
            <p class="setting-name" id="label-log-dir">Log Directory</p>
            <p class="setting-desc">Where per-task log files are written</p>
          </div>
          <input class="input input-sm" id="setting-log-dir" type="text"
            aria-labelledby="label-log-dir"
            bind:value={draft.log_dir} />
        </div>
        <div class="setting-row">
          <div class="setting-label-group">
            <p class="setting-name" id="label-log-size">Max File Size (MB)</p>
            <p class="setting-desc">Rotate a log file once it exceeds this size</p>
          </div>
          <input class="input input-sm" id="setting-log-size" type="number" min="1" max="1024"
            aria-labelledby="label-log-size"
            bind:value={draft.log_max_file_size_mb} />
        </div>
        <div class="setting-row">
          <div class="setting-label-group">
            <p class="setting-name" id="label-log-rotated">Max Rotated Files</p>
            <p class="setting-desc">How many rotated log files to keep per task</p>
          </div>
          <input class="input input-sm" id="setting-log-rotated" type="number" min="0" max="100"
            aria-labelledby="label-log-rotated"
            bind:value={draft.log_max_rotated_files} />
        </div>
      </section>

      <!-- Notifications -->
      <section class="settings-section">
        <h3 class="section-heading">Notifications</h3>
        <div class="setting-row">
          <div class="setting-label-group">
            <p class="setting-name" id="label-notifications">Desktop Notifications</p>
            <p class="setting-desc">Notify when a task crashes or stops</p>
          </div>
          <label class="toggle" aria-labelledby="label-notifications">
            <input type="checkbox" bind:checked={draft.notifications_enabled} />
            <span class="toggle-track"></span>
          </label>
        </div>
        <div class="setting-row">
          <div class="setting-label-group">
            <p class="setting-name" id="label-launch-startup">Launch at login</p>
            <p class="setting-desc">Start on system startup</p>
          </div>
          <label class="toggle" aria-labelledby="label-launch-startup">
            <input type="checkbox" bind:checked={draft.launch_on_startup} />
            <span class="toggle-track"></span>
          </label>
        </div>
      </section>

      <!-- Updates -->
      <section class="settings-section">
        <h3 class="section-heading">Updates</h3>
        <div class="setting-row">
          <div class="setting-label-group">
            <p class="setting-name" id="label-auto-updates">Auto-check for Updates</p>
          </div>
          <label class="toggle" aria-labelledby="label-auto-updates">
            <input type="checkbox" bind:checked={draft.auto_check_updates} />
            <span class="toggle-track"></span>
          </label>
        </div>

        <div class="update-check-area">
          <button class="btn" onclick={checkForUpdates} disabled={checkingUpdate || saving}>
            {checkingUpdate ? 'Checking…' : 'Check for Updates Now'}
          </button>

          {#if updateInfo}
            {#if updateInfo.available}
              <div class="update-available">
                <p class="update-label">Update available: <strong>{updateInfo.latest_version}</strong></p>
                {#if $selfUpdate && $installPhase !== 'error'}
                  <button class="btn btn-primary" onclick={installSelfUpdate} disabled={$installBusy}>
                    {$installBusy ? 'Working…' : 'Install Update'}
                  </button>
                {:else if updateInfo.release_url}
                  <a href={updateInfo.release_url} target="_blank" class="btn btn-primary">
                    View Release
                  </a>
                {/if}
              </div>
              {#if $selfUpdate && $installPhase !== 'error'}
                <p class="update-hint">Installing stops every running task and restarts Labalaba.</p>
              {/if}
              <UpdateProgress />
            {:else}
              <p class="update-current">You're on the latest version ({updateInfo.current_version})</p>
            {/if}
          {/if}

          {#if updateError}
            <p class="update-error">{updateError}</p>
          {/if}
        </div>
      </section>

      <!-- Danger Zone -->
      <section class="settings-section danger-zone">
        <h3 class="section-heading danger-heading">Danger Zone</h3>
        <p class="setting-desc danger-desc">
          Removing the background service stops the daemon and removes its autostart entry.
          The app keeps running but schedules and auto-restart won't work until you start the daemon again.
        </p>

        <div class="setting-row">
          <div class="setting-label-group">
            <p class="setting-name" id="label-purge-data">Also delete all tasks, settings, and logs</p>
            <p class="setting-desc danger-note">This permanently erases your data and cannot be undone.</p>
          </div>
          <label class="toggle" aria-labelledby="label-purge-data">
            <input type="checkbox" bind:checked={purgeData} disabled={cleaning} />
            <span class="toggle-track"></span>
          </label>
        </div>

        {#if cleanupDone}
          <p class="cleanup-done">{cleanupDone}</p>
        {/if}

        {#if cleanupError}
          <p class="cleanup-error">{cleanupError}</p>
        {/if}

        <div class="danger-action">
          <button
            class="btn btn-danger"
            onclick={() => { showCleanupConfirm = true; cleanupDone = null; cleanupError = null; }}
            disabled={cleaning || saving}
          >
            Remove Background Service
          </button>
        </div>
      </section>
    </div>

    <div class="modal-footer">
      {#if saveError}
        <p class="save-error">{saveError}</p>
      {/if}
      <button type="button" class="btn" onclick={onClose} disabled={saving}>Cancel</button>
      <button type="button" class="btn btn-primary" onclick={handleSave} disabled={saving}>
        {saving ? 'Saving…' : 'Save Settings'}
      </button>
    </div>
  </div>
</div>

{#if showPortableConfirm && dataLocation}
  <ConfirmDialog
    variant="warning"
    title={dataLocation.portable_active
      ? 'Move data back to your user profile?'
      : 'Keep data next to the app?'}
    message={portableConfirmMessage(dataLocation)}
    confirmText={dataLocation.portable_active ? 'Move data back' : 'Move data'}
    onConfirm={applyPortableToggle}
    onCancel={() => { showPortableConfirm = false; }}
  />
{/if}

{#if showCleanupConfirm}
  <ConfirmDialog
    variant="danger"
    title="Remove background service?"
    message={purgeData
      ? 'This stops the daemon, removes autostart, and PERMANENTLY DELETES all tasks, settings, and logs. This cannot be undone.'
      : 'This stops the daemon and removes its autostart entry. Your tasks and settings are kept.'}
    confirmText={purgeData ? 'Delete everything' : 'Remove service'}
    onConfirm={runCleanup}
    onCancel={() => { showCleanupConfirm = false; }}
  />
{/if}

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
    flex-wrap: wrap;
  }

  .setting-label-group {
    flex: 1 1 auto;
    min-width: 0;
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

  /* Paths are long and matter character-by-character: keep them monospace and
     let them wrap rather than truncating the part the user is looking for. */
  .setting-path {
    font-family: ui-monospace, SFMono-Regular, 'SF Mono', Menlo, Consolas, monospace;
    font-size: 0.75rem;
    color: var(--text-primary);
    margin-top: 0.25rem;
    overflow-wrap: anywhere;
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

  .update-label { font-size: 0.875rem; color: var(--accent); }
  .update-hint { font-size: 0.8125rem; color: var(--text-muted); margin: 0; }
  .update-current { font-size: 0.8125rem; color: var(--text-muted); }
  .update-error { font-size: 0.8125rem; color: var(--status-crashed); }

  .modal-footer {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 0.5rem;
    margin-top: 1.25rem;
    padding-top: 1rem;
    border-top: 1px solid var(--border-subtle);
  }

  .save-error {
    margin-right: auto;
    font-size: 0.8125rem;
    color: var(--status-crashed);
  }

  .danger-zone {
    border: 1px solid color-mix(in srgb, var(--status-crashed) 30%, transparent);
    border-radius: 6px;
    padding: 0.75rem;
    margin-top: 0.5rem;
  }

  .danger-heading {
    color: var(--status-crashed);
  }

  .danger-desc {
    margin-bottom: 0.5rem;
  }

  .danger-note {
    color: var(--status-crashed);
  }

  .danger-action {
    padding-top: 0.5rem;
  }

  .cleanup-done {
    font-size: 0.8125rem;
    color: var(--status-running, #4ade80);
    margin-top: 0.25rem;
  }

  .cleanup-error {
    font-size: 0.8125rem;
    color: var(--status-crashed);
    margin-top: 0.25rem;
  }
</style>
