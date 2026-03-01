<script lang="ts">
  import '../styles/theme.css';
  import '../styles/glassmorphism.css';
  import { theme } from '$lib/stores/theme';
  import { onMount } from 'svelte';

  let { children } = $props();

  onMount(() => {
    // Apply stored theme on mount
    document.documentElement.setAttribute('data-theme', $theme);
  });
</script>

<div class="app-shell" data-theme={$theme}>
  {@render children()}
</div>

<style>
  :global(*, *::before, *::after) {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
  }

  :global(body) {
    font-family: 'Inter', system-ui, -apple-system, sans-serif;
    font-size: 14px;
    line-height: 1.5;
    color: var(--text-primary);
    background: var(--bg-base);
    min-height: 100vh;
    overflow: hidden;
  }

  :global(button) { font-family: inherit; }
  :global(input, textarea, select) { font-family: inherit; }

  .app-shell {
    min-height: 100vh;
    background: var(--bg-base);
    /* Subtle radial gradient for depth */
    background-image:
      radial-gradient(ellipse 80% 60% at 20% 0%, rgba(99 179 237 / 0.07) 0%, transparent 60%),
      radial-gradient(ellipse 60% 50% at 80% 100%, rgba(159 122 234 / 0.05) 0%, transparent 60%);
  }
</style>
