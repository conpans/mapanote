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
  pinned: boolean;  // ← NEW
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

// ← NEW: Request types for adding/updating notes
export interface AddNoteRequest {
  country_slug: string;
  text: string;
  tags: string[];
  also: string[];
  visibility: 'private' | 'internal' | 'publishable';
  pinned: boolean;
}

export interface UpdateNoteRequest {
  country_slug: string;
  note_id: string;
  text: string;
  tags: string[];
  also: string[];
  visibility: 'private' | 'internal' | 'publishable';
  pinned: boolean;
}