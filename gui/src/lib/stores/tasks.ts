import { writable, derived } from 'svelte/store';
import { api, type TaskDto, type TaskStats } from '$lib/api/client';

export const tasks = writable<TaskDto[]>([]);
export const tasksLoading = writable(false);
export const tasksError = writable<string | null>(null);

export const stats = derived(tasks, ($tasks): TaskStats => ({
  total: $tasks.length,
  running: $tasks.filter(t => t.status === 'running' || t.status === 'starting').length,
  stopped: $tasks.filter(t => t.status === 'stopped').length,
  crashed: $tasks.filter(t => t.status === 'crashed').length,
}));

export async function loadTasks() {
  tasksLoading.set(true);
  tasksError.set(null);
  try {
    const list = await api.tasks.list();
    tasks.set(list);
  } catch (e) {
    tasksError.set(String(e));
  } finally {
    tasksLoading.set(false);
  }
}

export async function refreshTask(id: string) {
  try {
    const updated = await api.tasks.get(id);
    tasks.update(list => list.map(t =>
      t.config.id['0'] === id ? updated : t
    ));
  } catch { /* ignore */ }
}

export async function removeTask(id: string) {
  await api.tasks.remove(id);
  tasks.update(list => list.filter(t => t.config.id['0'] !== id));
}

// Poll for status updates every 2 seconds
export function startPolling(): () => void {
  const interval = setInterval(loadTasks, 2000);
  return () => clearInterval(interval);
}
