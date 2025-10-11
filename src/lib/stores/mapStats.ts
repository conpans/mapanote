import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

export interface CountryStats {
  slug: string;
  noteCount: number;
  lastUpdated: string | null;
}

export const mapStats = writable<Map<string, CountryStats>>(new Map());

/**
 * Load stats for all countries in the vault
 */
export async function loadMapStats(): Promise<void> {
  try {
    // Get list of all countries
    const countrySlugs = await invoke<string[]>('list_countries');
    
    const statsMap = new Map<string, CountryStats>();
    
    // Load note count for each country
    for (const slug of countrySlugs) {
      try {
        const notes = await invoke<any[]>('get_country_notes', { slug });
        
        // Find most recent note date
        let lastUpdated: string | null = null;
        if (notes.length > 0) {
          const dates = notes.map(n => n.date).sort();
          lastUpdated = dates[dates.length - 1];
        }
        
        statsMap.set(slug, {
          slug,
          noteCount: notes.length,
          lastUpdated,
        });
      } catch (err) {
        console.error(`Failed to load stats for ${slug}:`, err);
      }
    }
    
    mapStats.set(statsMap);
  } catch (error) {
    console.error('Failed to load map stats:', error);
    throw error;
  }
}

/**
 * Get activity level for a country (for color coding)
 */
export function getActivityLevel(noteCount: number): 'none' | 'low' | 'medium' | 'high' | 'very-high' {
  if (noteCount === 0) return 'none';
  if (noteCount <= 5) return 'low';
  if (noteCount <= 20) return 'medium';
  if (noteCount <= 50) return 'high';
  return 'very-high';
}

/**
 * Get freshness (notes in last 7 days)
 */
export function isFresh(lastUpdated: string | null): boolean {
  if (!lastUpdated) return false;
  
  const lastDate = new Date(lastUpdated);
  const now = new Date();
  const daysDiff = (now.getTime() - lastDate.getTime()) / (1000 * 60 * 60 * 24);
  
  return daysDiff <= 7;
}