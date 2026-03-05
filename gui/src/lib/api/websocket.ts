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

/** Fetches historical logs from the daemon via HTTP */
export async function fetchHistoricalLogs(
  taskId: string,
  lines: number = 500,
): Promise<LogEntry[]> {
  try {
    const response = await fetch(`http://127.0.0.1:${daemonPort}/api/logs/${taskId}?lines=${lines}`);
    const data = await response.json();
    
    if (data.success && data.data?.logs) {
      return data.data.logs;
    }
    
    return [];
  } catch (error) {
    console.error('Failed to fetch historical logs:', error);
    return [];
  }
}
