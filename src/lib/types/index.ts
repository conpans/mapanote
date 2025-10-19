export interface Country {
  slug: string;
  name: string;
  iso2: string;
  iso3: string;
  summary: string;
  region: string;
  subregion: string;
  note_count: number;        
  last_updated?: string;     
  tags: string[];            
}
export interface CountryMetadata {
  slug: string;
  name: string;
}

export interface Note {
  id: string;
  title: string;
  content: string;
  date: string;
  tags: string[];
  topic_id?: string;           // ← ADD
  country_targets: string[];   // ← ADD
}

export interface NoteWithSource {
  id: string;
  title: string;
  content: string;
  date: string;
  tags: string[];
  topic_id?: string;
  country_targets: string[];
  source_type: string;         
  source_name: string;         
  topic_color?: string;        
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

export interface RecentActivity {
  note_id: string;
  note_title: string;
  note_date: string;
  country_slug: string;
  country_name: string;
  source_type: string; // "country" or "topic"
  topic_name?: string;
  topic_color?: string;
}