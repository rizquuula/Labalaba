import { writable } from 'svelte/store';
import { api, type AppSettings } from '$lib/api/client';

export const settings = writable<AppSettings>({
  theme: 'dark',
  daemon_port: 27015,
  log_buffer_lines: 5000,
  config_path: './tasks.yaml',
  notifications_enabled: true,
  auto_check_updates: true,
  update_check_interval_hours: 24,
  launch_on_startup: false,
  log_dir: './logs',
  log_max_file_size_mb: 10,
  log_max_rotated_files: 5,
});

export const settingsLoading = writable(false);

export async function loadSettings() {
  settingsLoading.set(true);
  try {
    const s = await api.settings.get();
    settings.set(s);
  } finally {
    settingsLoading.set(false);
  }
}

export async function saveSettings(s: AppSettings) {
  settingsLoading.set(true);
  try {
    const updated = await api.settings.update(s);
    settings.set(updated);
  } finally {
    settingsLoading.set(false);
  }
}
