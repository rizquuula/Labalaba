import { invoke } from '@tauri-apps/api/core';

export interface TaskConfig {
  id: string;
  description: string;
  executable: string;
  arguments: string[];
  working_directory?: string;
  environment: Record<string, string>;
  run_as_admin: boolean;
  auto_restart: boolean;
  schedule?: { cron: string };
  startup_delay_ms: number;
  depends_on: string[];
  runner_prefix?: string;
  pids: number[];
}

export interface TaskDto {
  config: TaskConfig;
  status: 'stopped' | 'starting' | 'running' | 'stopping' | 'crashed';
  pid?: number;
  pids: number[];
  started_at?: string;
  exit_code?: number;
  cpu_percent?: number;
  memory_bytes?: number;
}

export interface TaskStats {
  total: number;
  running: number;
  stopped: number;
  crashed: number;
}

export interface TaskResourceStats {
  cpu_percent: number;
  memory_bytes: number;
}

export interface AppSettings {
  theme: string;
  daemon_port: number;
  log_buffer_lines: number;
  config_path: string;
  notifications_enabled: boolean;
  auto_check_updates: boolean;
  update_check_interval_hours: number;
  launch_on_startup: boolean;
  log_dir: string;
  log_max_file_size_mb: number;
  log_max_rotated_files: number;
}

export interface UpdateInfo {
  available: boolean;
  current_version: string;
  latest_version?: string;
  release_url?: string;
  release_notes?: string;
}

export interface TaskRequest {
  description: string;
  executable: string;
  arguments: string[];
  working_directory?: string;
  environment: Record<string, string>;
  run_as_admin: boolean;
  auto_restart: boolean;
  schedule?: { cron: string };
  startup_delay_ms: number;
  depends_on?: string[];
  runner_prefix?: string;
  pids?: number[];
}

export interface DaemonConnection {
  base_url: string;
  ws_url: string;
  token: string;
}

interface ApiResponse<T> {
  success: boolean;
  data: T | null;
  error: string | null;
}

let connectionPromise: Promise<DaemonConnection> | null = null;

export function getConnection(): Promise<DaemonConnection> {
  if (!connectionPromise) {
    // Don't cache a rejected promise: if the daemon isn't ready yet, clear the
    // cache so the next call retries instead of failing for the whole session.
    connectionPromise = invoke<DaemonConnection>('get_daemon_connection').catch((e) => {
      connectionPromise = null;
      throw e;
    });
  }
  return connectionPromise;
}

async function apiFetch<T>(method: string, path: string, body?: unknown): Promise<T> {
  const conn = await getConnection();
  const headers: Record<string, string> = {
    Authorization: `Bearer ${conn.token}`,
  };
  if (body !== undefined) {
    headers['content-type'] = 'application/json';
  }

  let response: Response;
  try {
    response = await fetch(conn.base_url + path, {
      method,
      headers,
      body: body !== undefined ? JSON.stringify(body) : undefined,
    });
  } catch (err) {
    throw new Error(`Network error: ${err}`);
  }

  const json: ApiResponse<T> = await response.json();
  if (!json.success) {
    throw new Error(json.error ?? 'request failed');
  }
  return json.data as T;
}

export const api = {
  tasks: {
    list: () => apiFetch<TaskDto[]>('GET', '/api/tasks'),
    get: (id: string) => apiFetch<TaskDto>('GET', `/api/tasks/${id}`),
    create: (req: TaskRequest) => apiFetch<TaskDto>('POST', '/api/tasks', req),
    update: (id: string, req: TaskRequest) => apiFetch<TaskDto>('PUT', `/api/tasks/${id}`, req),
    remove: (id: string) => apiFetch<null>('DELETE', `/api/tasks/${id}`),
    start: (id: string) => apiFetch<number>('POST', `/api/tasks/${id}/start`),
    stop: (id: string) => apiFetch<null>('POST', `/api/tasks/${id}/stop`),
    restart: (id: string) => apiFetch<number>('POST', `/api/tasks/${id}/restart`),
    getStats: (id: string) => apiFetch<TaskResourceStats>('GET', `/api/tasks/${id}/stats`),
  },
  stats: () => apiFetch<TaskStats>('GET', '/api/stats'),
  settings: {
    get: () => apiFetch<AppSettings>('GET', '/api/settings'),
    update: (settings: AppSettings) => apiFetch<AppSettings>('PUT', '/api/settings', settings),
  },
  update: {
    check: () => apiFetch<UpdateInfo>('POST', '/api/update/check'),
    pending: () => apiFetch<UpdateInfo | null>('GET', '/api/update/pending'),
  },
  system: {
    detectInterpreter: (kind: 'sh' | 'bash' | 'zsh' | 'ps1' | 'bat') =>
      apiFetch<string | null>('POST', '/api/system/detect-interpreter', { kind }),
  },
};

export function taskId(task: TaskDto): string {
  return task.config.id;
}
