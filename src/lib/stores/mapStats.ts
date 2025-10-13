import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

export interface CountryStats {
  slug: string;
  noteCount: number;
  lastUpdated: string | null;
}

export const mapStats = writable<Map<string, CountryStats>>(new Map());
export const mapStatsLoaded = writable<boolean>(false);  // ← ADD THIS

/**
 * Load stats for all countries in one efficient call
 */
export async function loadMapStats(): Promise<void> {
  try {
    const allStats = await invoke<CountryStats[]>('get_all_country_stats');
    
    const statsMap = new Map<string, CountryStats>();
    
    allStats.forEach(stat => {
      statsMap.set(stat.slug, {
        slug: stat.slug,
        noteCount: stat.noteCount,
        lastUpdated: stat.lastUpdated,
      });
    });
    
    mapStats.set(statsMap);
    mapStatsLoaded.set(true);  // ← ADD THIS
    
    console.log(`Loaded stats for ${statsMap.size} countries in one call`);
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