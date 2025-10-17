import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { loadMapStats } from './mapStats';
import { loadTopics } from './topics';
import type { Country, Note, NoteWithSource } from "$lib/types"; 

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
export const countries = writable<Country[]>([]); 
export const currentCountry = writable<CountryMetadata | null>(null);
export const currentNotes = writable<Note[]>([]);
export const currentNotesWithSource = writable<NoteWithSource[]>([]); 

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
    
    // Load full country metadata instead of just slugs
    const metadata = await invoke<Country[]>('get_all_countries_with_combined_counts');
    
    vaultPath.set(path);
    countries.set(metadata);
    vaultOpened.set(true);
    
    console.log('Loading initial map stats...');
    await loadMapStats();
    console.log('Map stats preloaded');
    
    console.log('Loading topics...');
    await loadTopics();
    console.log('Topics loaded');
    
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
    const metadata = await invoke<CountryMetadata>('get_country_metadata', { slug });
    currentCountry.set(metadata);
    
    // Use the new command that includes topic notes
    const notesWithSource = await invoke<NoteWithSource[]>('get_country_notes_with_topics', { slug });
    
    // Store the full data
    currentNotesWithSource.set(notesWithSource);
    
    // Extract just the notes for backward compatibility
    const notes = notesWithSource.map(nws => ({
      id: nws.id,
      title: nws.title,
      content: nws.content,
      date: nws.date,
      tags: nws.tags,
      topic_id: nws.topic_id,
      country_targets: nws.country_targets,
    }));
    currentNotes.set(notes);
    
  } catch (error) {
    console.error('Failed to load country:', error);
    currentCountry.set(null);
    currentNotes.set([]);
    currentNotesWithSource.set([]);
    throw error;
  } finally {
    isLoading.set(false);
  }
}

export async function loadCountries(): Promise<void> {
  isLoading.set(true);
  
  try {
    const data = await invoke<Country[]>('get_all_countries_with_combined_counts');
    countries.set(data);
  } catch (error) {
    console.error('Failed to load countries:', error);
    countries.set([]);
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