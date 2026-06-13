<script lang="ts">
  import { focusTrap } from '$lib/actions/focusTrap';

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

<div class="modal-backdrop" role="dialog" aria-modal="true" aria-labelledby="confirm-dialog-title" use:focusTrap={{ onClose: onCancel }}>
  <div class="modal glass-strong">
    <div class="modal-header">
      <h2 id="confirm-dialog-title">{title}</h2>
      <button class="btn-icon" aria-label="Close" onclick={onCancel} disabled={confirming}>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true" focusable="false">
          <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
        </svg>
      </button>
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
        {confirming ? confirmText + '…' : confirmText}
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
    border-top: 1px solid var(--border-subtle);
    padding-top: 1rem;
  }

  .btn-warning {
    color: var(--status-starting);
    border-color: var(--status-starting);
  }

  .btn-warning:hover {
    background: var(--status-starting);
    color: var(--bg-base);
  }

  .btn-danger {
    color: var(--status-crashed);
    border-color: var(--status-crashed);
  }

  .btn-danger:hover {
    background: var(--status-crashed);
    color: var(--bg-base);
  }
</style>
