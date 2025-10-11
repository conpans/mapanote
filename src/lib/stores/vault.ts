import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';  // ← Correct path for Tauri v2
import type { Country, Note } from '$lib/types';

// Vault state
export const vaultOpened = writable<boolean>(false);
export const vaultPath = writable<string>('');
export const countries = writable<string[]>([]);
export const currentCountry = writable<Country | null>(null);
export const currentNotes = writable<Note[]>([]);

// Derived store: is vault loading?
export const isLoading = writable<boolean>(false);

/**
 * Open a vault folder
 */
export async function openVault(path: string): Promise<void> {
  isLoading.set(true);
  
  try {
    // Call Rust command
    const result = await invoke<string>('open_vault', { path });
    console.log('Vault opened:', result);
    
    // Load list of countries
    const countryList = await invoke<string[]>('list_countries');
    
    // Update stores
    vaultPath.set(path);
    countries.set(countryList);
    vaultOpened.set(true);
    
  } catch (error) {
    console.error('Failed to open vault:', error);
    throw error;
  } finally {
    isLoading.set(false);
  }
}

/**
 * Load a country's data
 */
export async function loadCountry(slug: string): Promise<void> {
  isLoading.set(true);
  
  try {
    // Load country metadata
    const country = await invoke<Country>('get_country', { slug });
    currentCountry.set(country);
    
    // Load country notes
    const notes = await invoke<Note[]>('get_country_notes', { slug });
    currentNotes.set(notes);
    
  } catch (error) {
    console.error(`Failed to load country ${slug}:`, error);
    throw error;
  } finally {
    isLoading.set(false);
  }
}

/**
 * Close the vault
 */
export function closeVault(): void {
  vaultOpened.set(false);
  vaultPath.set('');
  countries.set([]);
  currentCountry.set(null);
  currentNotes.set([]);
}