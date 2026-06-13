import { getConnection, fetchWithTimeout } from './client';

export interface LogEntry {
  task_id: string;
  timestamp: string;
  stream: 'stdout' | 'stderr';
  line: string;
}

export type LogListener = (entry: LogEntry) => void;

/**
 * Subscribes to real-time log events for a task over WebSocket with auto-reconnect.
 * Awaits the first socket being created before resolving, so the caller can guarantee
 * no lines are missed between the history fetch and the listener attaching.
 * Returns a cleanup function; calling it stops reconnection and closes the socket.
 */
export async function connectLogStream(
  taskId: string,
  onEntry: LogListener,
  onClose?: () => void,
): Promise<() => void> {
  const conn = await getConnection();
  const url = `${conn.ws_url}/ws/logs/${taskId}?token=${encodeURIComponent(conn.token)}`;

  let socket: WebSocket | null = null;
  let stopped = false;
  let reconnectDelay = 1000;

  function connect(): Promise<void> {
    return new Promise((resolve) => {
      const ws = new WebSocket(url);
      socket = ws;

      ws.onopen = () => {
        reconnectDelay = 1000;
        resolve();
      };

      ws.onmessage = (event) => {
        try {
          const entry: LogEntry = JSON.parse(event.data as string);
          onEntry(entry);
        } catch {
          // ignore unparseable messages
        }
      };

      ws.onclose = () => {
        socket = null;
        // resolve in case open never fired (e.g. connection refused on first attempt)
        resolve();
        if (!stopped) {
          const delay = reconnectDelay;
          reconnectDelay = Math.min(reconnectDelay * 2, 10000);
          setTimeout(() => {
            if (!stopped) {
              connect();
            }
          }, delay);
        }
      };

      ws.onerror = () => {
        // onclose fires after onerror — let that drive reconnect
      };
    });
  }

  await connect();

  return () => {
    stopped = true;
    socket?.close();
    onClose?.();
  };
}

/**
 * Fetches a page of historical logs via the HTTP API. `offset` is the number of
 * newest lines to skip — `offset = 0` returns the most recent `lines`, and
 * increasing it walks backwards through history for a "load older" pager.
 */
export async function fetchHistoricalLogs(
  taskId: string,
  lines: number = 500,
  offset: number = 0,
): Promise<LogEntry[]> {
  try {
    const conn = await getConnection();
    const response = await fetchWithTimeout(
      `${conn.base_url}/api/logs/${taskId}?lines=${lines}&offset=${offset}`,
      { headers: { Authorization: `Bearer ${conn.token}` } },
    );
    const json: { success: boolean; data: { logs: LogEntry[] } | null; error: string | null } =
      await response.json();
    if (!json.success || !json.data) {
      return [];
    }
    return json.data.logs ?? [];
  } catch (error) {
    console.error('Failed to fetch historical logs:', error);
    return [];
  }
}
