import { writable, derived, get } from 'svelte/store';
import { check, type Update } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';
import { invoke } from '@tauri-apps/api/core';
import { loadTasks } from './tasks';

export type InstallPhase = 'idle' | 'downloading' | 'installing' | 'error';

// The daemon tells us *that* an update exists (GithubUpdater → /api/update/*);
// this store owns the separate question of whether we can install it ourselves
// and the act of doing so. It lives here rather than in a component because two
// unrelated surfaces offer the action — the update modal and the Settings panel
// — and they are never mounted at the same time. Shipping the install flow in
// only one of them is exactly the bug this replaces.

/** The pending update, or null when this platform can't install in place. */
export const selfUpdate = writable<Update | null>(null);
export const installPhase = writable<InstallPhase>('idle');
export const downloaded = writable(0);
export const contentLength = writable(0);
export const installError = writable<string | null>(null);

export const installBusy = derived(
  installPhase,
  ($p) => $p === 'downloading' || $p === 'installing'
);

export const downloadPct = derived([downloaded, contentLength], ([$d, $c]) =>
  $c > 0 ? Math.min(100, Math.round(($d / $c) * 100)) : 0
);

/**
 * Ask the updater plugin whether this build can update itself in place.
 *
 * A failure is expected and not surfaced: on a .deb/.rpm install, an Intel Mac,
 * or any release without a signed manifest for this platform, `check()` throws
 * or returns null and callers fall back to linking at the release page.
 */
export async function probeSelfUpdate(): Promise<void> {
  try {
    selfUpdate.set(await check());
  } catch (e) {
    console.warn('Self-update unavailable; falling back to manual download:', e);
    selfUpdate.set(null);
  }
}

/** Clear transient install state so a reopened surface offers Install again. */
export function resetInstallState(): void {
  installPhase.set('idle');
  installError.set(null);
  downloaded.set(0);
  contentLength.set(0);
}

export async function installSelfUpdate(): Promise<void> {
  const update = get(selfUpdate);
  if (!update || get(installBusy)) return;

  installError.set(null);
  downloaded.set(0);
  contentLength.set(0);
  installPhase.set('downloading');

  let daemonStopped = false;
  try {
    await update.download((e) => {
      switch (e.event) {
        case 'Started':
          contentLength.set(e.data.contentLength ?? 0);
          break;
        case 'Progress':
          downloaded.update((n) => n + e.data.chunkLength);
          break;
      }
    });

    // download() verifies the signature before resolving, so the update is both
    // present and trusted by this point. Only now is it safe to stop the daemon:
    // doing it earlier would kill the user's running tasks for an update that
    // might never have arrived.
    installPhase.set('installing');
    await invoke('prepare_for_update');
    daemonStopped = true;
    await update.install();

    // Windows exits inside install() and never reaches this line; macOS and
    // Linux install in place and need the restart requested explicitly.
    await relaunch();
  } catch (e) {
    installPhase.set('error');
    installError.set(e instanceof Error ? e.message : String(e));

    // If we got as far as stopping the daemon, the user's tasks are down and
    // the app is talking to nothing. A failed update is recoverable; leaving
    // them with a dead process manager is not.
    if (daemonStopped) {
      try {
        await invoke('start_daemon');
        await loadTasks();
      } catch (restartErr) {
        installError.update(
          (m) => `${m} — and the daemon could not be restarted: ${restartErr}. Restart Labalaba.`
        );
      }
    }
  }
}
