import { writable } from 'svelte/store';

export const showSearchModal = writable(false);

export function openSearch() {
  showSearchModal.set(true);
}

export function closeSearch() {
  showSearchModal.set(false);
}