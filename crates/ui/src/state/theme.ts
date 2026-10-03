import { createSignal, createEffect, onMount } from 'solid-js';

export function useTheme() {
  const [theme, setTheme] = createSignal<'light' | 'dark' | 'system'>('system');

  const toggleTheme = () => {
    setTheme(prev => prev === 'dark' ? 'light' : prev === 'light' ? 'system' : 'dark');
  };

  const applyTheme = () => {
    const root = document.documentElement;
    if (theme() === 'system') {
      root.setAttribute('data-theme', window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light');
    } else {
      root.setAttribute('data-theme', theme());
    }
  };

  createEffect(() => {
    applyTheme();
  });

  onMount(() => {
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    mediaQuery.addEventListener('change', applyTheme);
    return () => mediaQuery.removeEventListener('change', applyTheme);
  });

  return {
    theme,
    setTheme,
    toggleTheme,
  };
}