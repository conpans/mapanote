import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import type { Country, Note, AddNoteRequest, UpdateNoteRequest } from '$lib/types';
import { loadMapStats } from './mapStats';  // ← ADD THIS

// Vault state
export const vaultOpened = writable<boolean>(false);
export const vaultPath = writable<string>('');
export const countries = writable<string[]>([]);
export const currentCountry = writable<Country | null>(null);
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
    
    // ← ADD: Load map stats once when vault opens
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
 * Load a country's data
 */
export async function loadCountry(slug: string): Promise<void> {
  isLoading.set(true);
  
  try {
    const country = await invoke<Country>('get_country', { slug });
    currentCountry.set(country);
    
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
export async function addNote(request: AddNoteRequest): Promise<Note> {
  try {
    const note = await invoke<Note>('add_note', { request });
    
    // Reload notes
    await loadCountry(request.country_slug);
    
    // ← ADD: Refresh map stats after adding note
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
export async function updateNote(request: UpdateNoteRequest): Promise<void> {
  try {
    await invoke('update_note', { request });
    
    // Reload notes
    await loadCountry(request.country_slug);
    
    // ← ADD: Refresh map stats after updating note
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
    
    // ← ADD: Refresh map stats after deleting note
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