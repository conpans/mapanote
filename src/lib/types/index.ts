// Core vault types
export interface Country {
  slug: string;
  title: string;
  region: string;
  summary: string;
  aliases: string[];
  updated_at: string;
}

export interface Note {
  id: string;
  date: string;
  tags: string[];
  text: string;
  also: string[];  // Cross-references
  visibility: 'private' | 'internal' | 'publishable';
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