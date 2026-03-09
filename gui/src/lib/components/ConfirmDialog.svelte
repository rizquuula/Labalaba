<script lang="ts">
  let {
    title,
    message,
    confirmText = 'Confirm',
    cancelText = 'Cancel',
    variant = 'warning',
    onConfirm,
    onCancel
  } = $props<{
    title: string;
    message: string;
    confirmText?: string;
    cancelText?: string;
    variant?: 'warning' | 'danger' | 'info';
    onConfirm: () => void | Promise<void>;
    onCancel: () => void;
  }>();

  let confirming = $state(false);

  async function handleConfirm() {
    confirming = true;
    try {
      await onConfirm();
    } finally {
      confirming = false;
    }
  }
</script>

<div class="modal-backdrop" role="dialog" aria-modal="true">
  <div class="modal glass-strong">
    <div class="modal-header">
      <h2>{title}</h2>
    </div>
    
    <p class="modal-message">{message}</p>
    
    <div class="modal-footer">
      <button class="btn" onclick={onCancel} disabled={confirming}>
        {cancelText}
      </button>
      <button 
        class="btn btn-{variant}" 
        onclick={handleConfirm} 
        disabled={confirming}
      >
        {confirming ? '…' : confirmText}
      </button>
    </div>
  </div>
</div>

<style>
  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 1rem;
  }

  .modal-header h2 {
    font-size: 1.0625rem;
    font-weight: 700;
    color: var(--text-primary);
  }

  .modal-message {
    font-size: 0.875rem;
    color: var(--text-primary);
    line-height: 1.6;
    margin-bottom: 1.5rem;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
  }

  .btn-warning {
    color: var(--status-running);
    border-color: var(--status-running);
  }

  .btn-warning:hover {
    background: var(--status-running);
    color: var(--bg-primary);
  }

  .btn-danger {
    color: var(--status-crashed);
    border-color: var(--status-crashed);
  }

  .btn-danger:hover {
    background: var(--status-crashed);
    color: var(--bg-primary);
  }
</style>
