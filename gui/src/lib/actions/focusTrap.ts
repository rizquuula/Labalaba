import type { Action } from 'svelte/action';

interface FocusTrapParams {
	/** Called when the user presses Escape inside the trapped region. */
	onClose?: () => void;
}

const FOCUSABLE_SELECTOR = [
	'a[href]',
	'button:not([disabled])',
	'input:not([disabled])',
	'select:not([disabled])',
	'textarea:not([disabled])',
	'[tabindex]:not([tabindex="-1"])'
].join(',');

/**
 * Modal focus management for Svelte 5: traps Tab/Shift+Tab inside `node`,
 * moves focus to the first focusable element on mount, closes on Escape,
 * and restores focus to the previously-focused element on destroy.
 *
 * Usage: `<div class="modal-backdrop" use:focusTrap={{ onClose }}> … </div>`
 */
export const focusTrap: Action<HTMLElement, FocusTrapParams | undefined> = (node, params) => {
	let options: FocusTrapParams = params ?? {};
	const previouslyFocused = document.activeElement as HTMLElement | null;

	const focusable = (): HTMLElement[] =>
		Array.from(node.querySelectorAll<HTMLElement>(FOCUSABLE_SELECTOR)).filter(
			(el) => el.offsetParent !== null || el === document.activeElement
		);

	const focusFirst = () => {
		const items = focusable();
		(items[0] ?? node).focus();
	};

	const handleKeydown = (event: KeyboardEvent) => {
		if (event.key === 'Escape') {
			event.stopPropagation();
			options.onClose?.();
			return;
		}
		if (event.key !== 'Tab') return;

		const items = focusable();
		if (items.length === 0) {
			event.preventDefault();
			return;
		}

		const first = items[0];
		const last = items[items.length - 1];
		const active = document.activeElement as HTMLElement | null;

		if (event.shiftKey && active === first) {
			event.preventDefault();
			last.focus();
		} else if (!event.shiftKey && active === last) {
			event.preventDefault();
			first.focus();
		}
	};

	if (!node.hasAttribute('tabindex')) {
		node.setAttribute('tabindex', '-1');
	}
	node.addEventListener('keydown', handleKeydown);
	queueMicrotask(focusFirst);

	return {
		update(newParams) {
			options = newParams ?? {};
		},
		destroy() {
			node.removeEventListener('keydown', handleKeydown);
			previouslyFocused?.focus?.();
		}
	};
};
