// HTTP client that talks to the labalaba-daemon REST API

export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
}

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
}

export interface TaskDto {
  config: TaskConfig;
  status: 'stopped' | 'starting' | 'running' | 'stopping' | 'crashed';
  pid?: number;
  started_at?: string;
  exit_code?: number;
}

export interface TaskStats {
  total: number;
  running: number;
  stopped: number;
  crashed: number;
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
}

let daemonPort = 27015;

export function setDaemonPort(port: number) {
  daemonPort = port;
}

function base(): string {
  return `http://127.0.0.1:${daemonPort}`;
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${base()}${path}`, {
    headers: { 'Content-Type': 'application/json' },
    ...init,
  });
  const body: ApiResponse<T> = await res.json();
  if (!body.success || body.data === undefined) {
    throw new Error(body.error ?? 'Unknown error');
  }
  return body.data;
}

export const api = {
  tasks: {
    list: () => request<TaskDto[]>('/api/tasks'),
    get: (id: string) => request<TaskDto>(`/api/tasks/${id}`),
    create: (req: TaskRequest) => request<TaskDto>('/api/tasks', {
      method: 'POST', body: JSON.stringify(req),
    }),
    update: (id: string, req: TaskRequest) => request<TaskDto>(`/api/tasks/${id}`, {
      method: 'PUT', body: JSON.stringify(req),
    }),
    remove: (id: string) => request<void>(`/api/tasks/${id}`, { method: 'DELETE' }),
    start: (id: string) => request<number>(`/api/tasks/${id}/start`, { method: 'POST' }),
    stop: (id: string) => request<void>(`/api/tasks/${id}/stop`, { method: 'POST' }),
    restart: (id: string) => request<number>(`/api/tasks/${id}/restart`, { method: 'POST' }),
  },
  stats: () => request<TaskStats>('/api/stats'),
  settings: {
    get: () => request<AppSettings>('/api/settings'),
    update: (s: AppSettings) => request<AppSettings>('/api/settings', {
      method: 'PUT', body: JSON.stringify(s),
    }),
  },
  update: {
    check: () => request<UpdateInfo>('/api/update/check', { method: 'POST' }),
  },
};

export function taskId(task: TaskDto): string {
  return task.config.id;
}
