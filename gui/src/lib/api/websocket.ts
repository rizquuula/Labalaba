// Tauri event listener — replaces the WebSocket layer for log streaming
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';

export interface LogEntry {
  task_id: string;
  timestamp: string;
  stream: 'stdout' | 'stderr';
  line: string;
}

export type LogListener = (entry: LogEntry) => void;

/** Subscribes to real-time log events for a task. Returns a cleanup function. */
export function connectLogStream(
  taskId: string,
  onEntry: LogListener,
  onClose?: () => void,
): () => void {
  let unlisten: (() => void) | undefined;

  listen<LogEntry>(`log:${taskId}`, (event) => {
    onEntry(event.payload);
  }).then((fn) => {
    unlisten = fn;
  });

  // Return cleanup — call the unlisten fn when available
  return () => {
    if (unlisten) {
      unlisten();
    }
    onClose?.();
  };
}

/** Fetches historical logs via Tauri command */
export async function fetchHistoricalLogs(
  taskId: string,
  lines: number = 500,
): Promise<LogEntry[]> {
  try {
    return await invoke<LogEntry[]>('get_logs', { id: taskId, lines });
  } catch (error) {
    console.error('Failed to fetch historical logs:', error);
    return [];
  }
}
