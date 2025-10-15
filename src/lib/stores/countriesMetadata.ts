import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

export interface CountryMetadata {
  slug: string;
  name: string;
  iso2: string;
  iso3: string;
  flag: string;
  summary: string;
  region: string;
  subregion: string;
}

export const countriesMetadata = writable<Map<string, CountryMetadata>>(new Map());

let loaded = false;

export async function loadCountriesMetadata() {
  if (loaded) return;
  
  try {
    const countries = await invoke<CountryMetadata[]>('get_all_countries_metadata');
    const metadataMap = new Map(countries.map(c => [c.slug, c]));
    countriesMetadata.set(metadataMap);
    loaded = true;
    console.log(`Loaded metadata for ${countries.length} countries`);
  } catch (error) {
    console.error('Failed to load countries metadata:', error);
  }
}