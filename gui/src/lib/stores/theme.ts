import { writable } from 'svelte/store';
import { browser } from '$app/environment';

type Theme = 'dark' | 'light';

const STORAGE_KEY = 'labalaba-theme';

function createThemeStore() {
  const initial: Theme = browser
    ? ((localStorage.getItem(STORAGE_KEY) as Theme) ?? 'dark')
    : 'dark';

  const { subscribe, set, update } = writable<Theme>(initial);

  if (browser) {
    document.documentElement.setAttribute('data-theme', initial);
  }

  return {
    subscribe,
    toggle() {
      update(t => {
        const next: Theme = t === 'dark' ? 'light' : 'dark';
        if (browser) {
          localStorage.setItem(STORAGE_KEY, next);
          document.documentElement.setAttribute('data-theme', next);
        }
        return next;
      });
    },
    set(t: Theme) {
      if (browser) {
        localStorage.setItem(STORAGE_KEY, t);
        document.documentElement.setAttribute('data-theme', t);
      }
      set(t);
    },
  };
}

export const theme = createThemeStore();
