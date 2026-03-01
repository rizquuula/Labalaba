// WebSocket client for real-time log streaming from the daemon

export interface LogEntry {
  task_id: string;
  timestamp: string;
  stream: 'stdout' | 'stderr';
  line: string;
}

export type LogListener = (entry: LogEntry) => void;

let daemonPort = 27015;

export function setWsDaemonPort(port: number) {
  daemonPort = port;
}

/** Opens a WebSocket for a task's log stream. Returns a cleanup function. */
export function connectLogStream(
  taskId: string,
  onEntry: LogListener,
  onClose?: () => void,
): () => void {
  const ws = new WebSocket(`ws://127.0.0.1:${daemonPort}/ws/logs/${taskId}`);

  ws.onmessage = (ev) => {
    try {
      const entry: LogEntry = JSON.parse(ev.data);
      onEntry(entry);
    } catch {
      // ignore malformed frames
    }
  };

  ws.onclose = () => onClose?.();
  ws.onerror = () => ws.close();

  return () => ws.close();
}
