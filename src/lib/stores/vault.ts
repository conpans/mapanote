import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import type { Note } from '$lib/types';
import { loadMapStats } from './mapStats';

// Country metadata from embedded data
export interface CountryMetadata {
  slug: string;
  name: string;
  iso2: string;
  iso3: string;
  summary: string;
  region: string;
  subregion: string;
}

// Vault state
export const vaultOpened = writable<boolean>(false);
export const vaultPath = writable<string>('');
export const countries = writable<string[]>([]);
export const currentCountry = writable<CountryMetadata | null>(null);
export const currentNotes = writable<Note[]>([]);

// UI state
export const isLoading = writable<boolean>(false);

/**
 * Open a vault folder
 */
export async function openVault(path: string): Promise<void> {
  isLoading.set(true);
  
  try {
    const result = await invoke<string>('open_vault', { path });
    console.log('Vault opened:', result);
    
    const countryList = await invoke<string[]>('list_countries');
    
    vaultPath.set(path);
    countries.set(countryList);
    vaultOpened.set(true);
    
    console.log('Loading initial map stats...');
    await loadMapStats();
    console.log('Map stats preloaded');
    
  } catch (error) {
    console.error('Failed to open vault:', error);
    throw error;
  } finally {
    isLoading.set(false);
  }
}

/**
 * Load a country's data from embedded metadata + vault notes
 */
export async function loadCountry(slug: string): Promise<void> {
  isLoading.set(true);
  
  try {
    // Load metadata from embedded JSON (always available)
    const metadata = await invoke<CountryMetadata>('get_country_metadata', { slug });
    currentCountry.set(metadata);
    
    // Load notes from vault (returns empty array if country folder doesn't exist yet)
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
 * Add a new note
 */
export async function addNote(
  countrySlug: string,
  title: string,
  content: string,
  tags: string[]
): Promise<Note> {
  try {
    const note = await invoke<Note>('add_note', {
      countrySlug,
      title,
      content,
      tags,
    });
    
    // Reload notes
    await loadCountry(countrySlug);
    
    // Refresh map stats after adding note
    await loadMapStats();
    
    return note;
  } catch (error) {
    console.error('Failed to add note:', error);
    throw error;
  }
}

/**
 * Update an existing note
 */
export async function updateNote(
  countrySlug: string,
  noteId: string,
  title: string,
  content: string,
  tags: string[]
): Promise<void> {
  try {
    await invoke('update_note', {
      countrySlug,
      noteId,
      title,
      content,
      tags,
    });
    
    // Reload notes
    await loadCountry(countrySlug);
    
    // Refresh map stats after updating note
    await loadMapStats();
    
  } catch (error) {
    console.error('Failed to update note:', error);
    throw error;
  }
}

/**
 * Delete a note
 */
export async function deleteNote(countrySlug: string, noteId: string): Promise<void> {
  try {
    await invoke('delete_note', { 
      countrySlug, 
      noteId 
    });
    
    // Reload notes
    await loadCountry(countrySlug);
    
    // Refresh map stats after deleting note
    await loadMapStats();
    
  } catch (error) {
    console.error('Failed to delete note:', error);
    throw error;
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