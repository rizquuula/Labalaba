<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import TopBar from '$lib/components/TopBar.svelte';
  import TaskList from '$lib/components/TaskList.svelte';
  import TaskForm from '$lib/components/TaskForm.svelte';
  import LogViewer from '$lib/components/LogViewer.svelte';
  import Settings from '$lib/components/Settings.svelte';
  import { loadTasks, startPolling, tasks } from '$lib/stores/tasks';
  import type { TaskDto } from '$lib/api/client';

  // Modal state
  let showForm = $state(false);
  let editingTask = $state<TaskDto | undefined>(undefined);
  let showSettings = $state(false);
  let activeLogTaskId = $state<string | null>(null);

  const activeLogTask = $derived(
    activeLogTaskId ? $tasks.find(t => t.config.id['0'] === activeLogTaskId) : null
  );

  let stopPolling: (() => void) | null = null;

  onMount(async () => {
    await loadTasks();
    stopPolling = startPolling();
  });

  onDestroy(() => stopPolling?.());

  function openAddForm() {
    editingTask = undefined;
    showForm = true;
  }

  function openEditForm(task: TaskDto) {
    editingTask = task;
    showForm = true;
  }

  function closeForm() {
    showForm = false;
    editingTask = undefined;
  }

  function openLogs(id: string) {
    activeLogTaskId = activeLogTaskId === id ? null : id;
  }

  function closeLogs() {
    activeLogTaskId = null;
  }
</script>

<div class="layout">
  <TopBar onSettingsClick={() => (showSettings = true)} />

  <main class="main-content">
    <TaskList onViewLogs={openLogs} onEdit={openEditForm} onAddNew={openAddForm} />

    {#if activeLogTask}
      <LogViewer
        taskId={activeLogTask.config.id[0]}
        taskName={activeLogTask.config.description}
        onClose={closeLogs}
      />
    {/if}
  </main>
</div>

{#if showForm}
  <TaskForm task={editingTask} onClose={closeForm} />
{/if}

{#if showSettings}
  <Settings onClose={() => (showSettings = false)} />
{/if}

<style>
  .layout {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
  }

  .main-content {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
  }
</style>
