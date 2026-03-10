// Tauri command client — replaces the HTTP fetch layer
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
}

export const api = {
  tasks: {
    list: () => invoke<TaskDto[]>('list_tasks'),
    get: (id: string) => invoke<TaskDto>('get_task', { id }),
    create: (req: TaskRequest) => invoke<TaskDto>('create_task', { req }),
    update: (id: string, req: TaskRequest) => invoke<TaskDto>('update_task', { id, req }),
    remove: (id: string) => invoke<void>('delete_task', { id }),
    start: (id: string) => invoke<number>('start_task', { id }),
    stop: (id: string) => invoke<void>('stop_task', { id }),
    restart: (id: string) => invoke<number>('restart_task', { id }),
    getStats: (id: string) => invoke<TaskResourceStats>('get_task_stats', { id }),
  },
  stats: () => invoke<TaskStats>('get_stats'),
  settings: {
    get: () => invoke<AppSettings>('get_settings'),
    update: (settings: AppSettings) => invoke<AppSettings>('update_settings', { settings }),
  },
  update: {
    check: () => invoke<UpdateInfo>('check_update'),
  },
};

export function taskId(task: TaskDto): string {
  return task.config.id;
}
