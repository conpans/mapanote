// Core vault types

export interface Country {
  slug: string;
  name: string;
  iso2: string;
  iso3: string;
  summary: string;
  region: string;
  subregion: string;
}

export interface Note {
  id: string;
  date: string;
  title: string;
  content: string;
  tags: string[];
}

export interface VaultConfig {
  schema_version: number;
  created_at: string;
  vault_path: string;
}

export interface SearchResult {
  note_id: string;
  country_slug: string;
  snippet: string;
  date: string;
  tags: string[];
}

// Map stats type
export interface CountryStats {
  slug: string;
  noteCount: number;
  lastUpdated: string | null;
  tags: string[];
}

export interface Topic {
  id: string;
  title: string;
  summary?: string;
  color?: string;
  pinned: boolean;
  created_at: string;
  updated_at: string;
}

export interface TopicWithCountries {
  id: string;
  title: string;
  summary?: string;
  color?: string;
  pinned: boolean;
  created_at: string;
  updated_at: string;
  countries: string[];
  note_count: number;
}

export interface TopicCountryRelation {
  topic_id: string;
  country_slug: string;
  note_count: number;
  last_updated?: string;
}