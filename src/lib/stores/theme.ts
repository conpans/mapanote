import { writable } from 'svelte/store';
import { browser } from '$app/environment';

type Theme = 'light' | 'dark';

// Initialize from localStorage or system preference
function getInitialTheme(): Theme {
  if (!browser) return 'light';
  
  const stored = localStorage.getItem('mapanote-theme');
  if (stored === 'light' || stored === 'dark') return stored;
  
  // Fallback to system preference
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

export const theme = writable<Theme>(getInitialTheme());

// Apply theme to document
export function applyTheme(newTheme: Theme) {
  if (!browser) return;
  
  if (newTheme === 'dark') {
    document.documentElement.classList.add('dark');
  } else {
    document.documentElement.classList.remove('dark');
  }
  
  localStorage.setItem('mapanote-theme', newTheme);
  theme.set(newTheme);
}

// Initialize on module load
if (browser) {
  applyTheme(getInitialTheme());
}

export function toggleTheme() {
  theme.update(current => {
    const next = current === 'light' ? 'dark' : 'light';
    applyTheme(next);
    return next;
  });
}