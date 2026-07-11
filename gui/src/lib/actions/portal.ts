import type { Action } from 'svelte/action';

/** Moves `node` to document.body (or a target) so fixed-position modals escape any
 *  ancestor stacking context / containing block created by transform/filter/backdrop-filter. */
export const portal: Action<HTMLElement, HTMLElement | string | undefined> = (node, target) => {
  const resolve = (t: HTMLElement | string | undefined): HTMLElement =>
    (typeof t === 'string' ? document.querySelector<HTMLElement>(t) : t) ?? document.body;
  let host = resolve(target);
  host.appendChild(node);
  return {
    update(newTarget) { host = resolve(newTarget); host.appendChild(node); },
    destroy() { node.parentNode?.removeChild(node); }
  };
};
