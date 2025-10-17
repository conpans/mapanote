import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import type { Country } from '$lib/types';

export interface CountryStats {
  slug: string;
  noteCount: number;
  lastUpdated: string | null;
  tags: string[];
}

export const mapStats = writable<Map<string, CountryStats>>(new Map());
export const mapStatsLoaded = writable<boolean>(false);

/**
 * Load stats for all countries with COMBINED counts (country + topic notes)
 */
export async function loadMapStats(): Promise<void> {
  try {
    // Use the NEW command that includes topic notes targeting each country
    const allCountries = await invoke<Country[]>('get_all_countries_with_combined_counts');
    
    const statsMap = new Map<CountryStats>();
    
    allCountries.forEach(country => {
      statsMap.set(country.slug, {
        slug: country.slug,
        noteCount: country.note_count,
        lastUpdated: country.last_updated ?? null,
        tags: country.tags,
      });
    });
    
    mapStats.set(statsMap);
    mapStatsLoaded.set(true);
    
    console.log(`✅ Loaded COMBINED stats for ${statsMap.size} countries`);
  } catch (error) {
    console.error('Failed to load map stats:', error);
    mapStatsLoaded.set(false);
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