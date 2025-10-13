use crate::vault::{VaultReader, VaultWriter};
use crate::models::{Country, Note, AddNoteRequest, UpdateNoteRequest};
use tauri::State;
use std::sync::Mutex;
use std::path::PathBuf;

/// App state shared across commands
pub struct AppState {
    pub vault_reader: Mutex<Option<VaultReader>>,
    pub vault_writer: Mutex<Option<VaultWriter>>,  // ← NEW
}

#[tauri::command]
pub async fn open_vault(path: String, state: State<'_, AppState>) -> Result<String, String> {
    let vault_path = PathBuf::from(&path);
    let vault_reader = VaultReader::new(vault_path.clone());
    let vault_writer = VaultWriter::new(vault_path);  // ← NEW
    
    // Verify vault exists by loading config
    match vault_reader.load_config() {
        Ok(config) => {
            *state.vault_reader.lock().unwrap() = Some(vault_reader);
            *state.vault_writer.lock().unwrap() = Some(vault_writer);  // ← NEW
            Ok(format!("Vault opened: schema v{}", config.schema_version))
        }
        Err(e) => Err(format!("Failed to open vault: {}", e)),
    }
}

#[tauri::command]
pub async fn list_countries(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let reader_guard = state.vault_reader.lock().unwrap();
    
    match reader_guard.as_ref() {
        Some(reader) => {
            reader.list_countries().map_err(|e| e.to_string())
        }
        None => Err("No vault opened".to_string()),
    }
}

#[tauri::command]
pub async fn get_country(slug: String, state: State<'_, AppState>) -> Result<Country, String> {
    let reader_guard = state.vault_reader.lock().unwrap();
    
    match reader_guard.as_ref() {
        Some(reader) => {
            let page = reader.read_country(&slug).map_err(|e| e.to_string())?;
            Ok(page.country)
        }
        None => Err("No vault opened".to_string()),
    }
}

#[tauri::command]
pub async fn get_country_notes(slug: String, state: State<'_, AppState>) -> Result<Vec<Note>, String> {
    let reader_guard = state.vault_reader.lock().unwrap();
    
    match reader_guard.as_ref() {
        Some(reader) => {
            let page = reader.read_country(&slug).map_err(|e| e.to_string())?;
            Ok(page.notes)
        }
        None => Err("No vault opened".to_string()),
    }
}

// ← NEW COMMANDS BELOW

#[tauri::command]
pub async fn add_note(request: AddNoteRequest, state: State<'_, AppState>) -> Result<Note, String> {
    let writer_guard = state.vault_writer.lock().unwrap();
    
    match writer_guard.as_ref() {
        Some(writer) => {
            writer.add_note(request).map_err(|e| e.to_string())
        }
        None => Err("No vault opened".to_string()),
    }
}

#[tauri::command]
pub async fn update_note(request: UpdateNoteRequest, state: State<'_, AppState>) -> Result<(), String> {
    let writer_guard = state.vault_writer.lock().unwrap();
    
    match writer_guard.as_ref() {
        Some(writer) => {
            writer.update_note(request).map_err(|e| e.to_string())
        }
        None => Err("No vault opened".to_string()),
    }
}

#[tauri::command]
pub async fn delete_note(country_slug: String, note_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let writer_guard = state.vault_writer.lock().unwrap();
    
    match writer_guard.as_ref() {
        Some(writer) => {
            writer.delete_note(&country_slug, &note_id).map_err(|e| e.to_string())
        }
        None => Err("No vault opened".to_string()),
    }
}


/// Search result combining note data with country info
#[derive(Debug, Clone, serde::Serialize)]
pub struct SearchResult {
    pub note_id: String,
    pub country_slug: String,
    pub country_title: String,
    pub date: String,
    pub tags: Vec<String>,
    pub text: String,
    pub snippet: String,  // Highlighted excerpt
    pub visibility: String,
    pub pinned: bool,
}

#[tauri::command]
pub async fn search_notes(
    query: String, 
    state: State<'_, AppState>
) -> Result<Vec<SearchResult>, String> {
    let reader_guard = state.vault_reader.lock().unwrap();
    
    match reader_guard.as_ref() {
        Some(reader) => {
            // Get all countries
            let countries = reader.list_countries().map_err(|e| e.to_string())?;
            
            let mut results = Vec::new();
            let query_lower = query.to_lowercase();
            
            // Search through each country
            for slug in countries {
                let page = match reader.read_country(&slug) {
                    Ok(p) => p,
                    Err(_) => continue,
                };
                
                // Search through notes
                for note in page.notes {
                    // Check if query matches text or tags
                    let text_lower = note.text.to_lowercase();
                    let tags_str = note.tags.join(" ").to_lowercase();
                    
                    if text_lower.contains(&query_lower) || tags_str.contains(&query_lower) {
                        // Create snippet (extract surrounding context)
                        let snippet = create_snippet(&note.text, &query, 100);
                        
                        results.push(SearchResult {
                            note_id: note.id.clone(),
                            country_slug: slug.clone(),
                            country_title: page.country.title.clone(),
                            date: note.date.clone(),
                            tags: note.tags.clone(),
                            text: note.text.clone(),
                            snippet,
                            visibility: format!("{:?}", note.visibility).to_lowercase(),
                            pinned: note.pinned,
                        });
                    }
                }
            }
            
            // Sort by date (newest first)
            results.sort_by(|a, b| b.date.cmp(&a.date));
            
            Ok(results)
        }
        None => Err("No vault opened".to_string()),
    }
}

/// Create a snippet with context around the search term
fn create_snippet(text: &str, query: &str, max_len: usize) -> String {
    let text_lower = text.to_lowercase();
    let query_lower = query.to_lowercase();
    
    if let Some(pos) = text_lower.find(&query_lower) {
        // Calculate start and end positions for snippet
        let start = pos.saturating_sub(max_len / 2);
        let end = (pos + query.len() + max_len / 2).min(text.len());
        
        let mut snippet = String::new();
        
        if start > 0 {
            snippet.push_str("...");
        }
        
        snippet.push_str(&text[start..end]);
        
        if end < text.len() {
            snippet.push_str("...");
        }
        
        snippet
    } else {
        // Fallback: just return first N chars
        let end = max_len.min(text.len());
        let mut snippet = text[..end].to_string();
        if end < text.len() {
            snippet.push_str("...");
        }
        snippet
    }
}

#[tauri::command]
pub async fn export_country_markdown(
    country_slug: String,
    state: State<'_, AppState>
) -> Result<String, String> {
    let reader_guard = state.vault_reader.lock().unwrap();
    
    match reader_guard.as_ref() {
        Some(reader) => {
            let page = reader.read_country(&country_slug).map_err(|e| e.to_string())?;
            
            // Build markdown content
            let mut content = String::new();
            
            // Add frontmatter
            content.push_str("---\n");
            content.push_str(&format!("title: {}\n", page.country.title));
            content.push_str(&format!("slug: {}\n", page.country.slug));
            content.push_str(&format!("region: {}\n", page.country.region));
            if !page.country.summary.is_empty() {
                content.push_str(&format!("summary: {}\n", page.country.summary));
            }
            if !page.country.aliases.is_empty() {
                content.push_str(&format!("aliases: [{}]\n", page.country.aliases.join(", ")));
            }
            content.push_str(&format!("updated_at: {}\n", page.country.updated_at));
            content.push_str("---\n\n");
            
            // Add overview
            content.push_str(&format!("# {}\n\n", page.country.title));
            content.push_str(&format!("**Region:** {}\n\n", page.country.region));
            if !page.country.summary.is_empty() {
                content.push_str(&format!("{}\n\n", page.country.summary));
            }
            
            // Add notes
            content.push_str("## Notes\n\n");
            for note in &page.notes {
                let mut metadata = vec![note.date.clone()];
                
                if !note.tags.is_empty() {
                    metadata.push(note.tags.join(", "));
                }
                
                if !note.also.is_empty() {
                    metadata.push(format!("also:{}", note.also.join(",")));
                }
                
                metadata.push(format!("{:?}", note.visibility).to_lowercase());
                
                if note.pinned {
                    metadata.push("pinned".to_string());
                }
                
                content.push_str(&format!("### {} · {}\n", note.date, metadata[1..].join(" · ")));
                content.push_str(&format!("[id:{}]\n\n", note.id));
                content.push_str(&format!("{}\n\n", note.text));
            }
            
            Ok(content)
        }
        None => Err("No vault opened".to_string()),
    }
}

#[tauri::command]
pub async fn get_vault_stats(state: State<'_, AppState>) -> Result<VaultStats, String> {
    let reader_guard = state.vault_reader.lock().unwrap();
    
    match reader_guard.as_ref() {
        Some(reader) => {
            let countries = reader.list_countries().map_err(|e| e.to_string())?;
            let mut total_notes = 0;
            let mut total_pinned = 0;
            let mut all_tags = std::collections::HashSet::new();
            
            for slug in &countries {
                if let Ok(page) = reader.read_country(slug) {
                    total_notes += page.notes.len();
                    total_pinned += page.notes.iter().filter(|n| n.pinned).count();
                    
                    for note in page.notes {
                        for tag in note.tags {
                            all_tags.insert(tag);
                        }
                    }
                }
            }
            
            Ok(VaultStats {
                country_count: countries.len(),
                note_count: total_notes,
                pinned_count: total_pinned,
                tag_count: all_tags.len(),
            })
        }
        None => Err("No vault opened".to_string()),
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct VaultStats {
    pub country_count: usize,
    pub note_count: usize,
    pub pinned_count: usize,
    pub tag_count: usize,
}

#[tauri::command]
pub async fn get_all_country_stats(state: State<'_, AppState>) -> Result<Vec<CountryStatsItem>, String> {
    let reader_guard = state.vault_reader.lock().unwrap();
    
    match reader_guard.as_ref() {
        Some(reader) => {
            let countries = reader.list_countries().map_err(|e| e.to_string())?;
            let mut stats = Vec::new();
            
            for slug in countries {
                if let Ok(page) = reader.read_country(&slug) {
                    let note_count = page.notes.len();
                    
                    // Find most recent note date
                    let last_updated = if !page.notes.is_empty() {
                        let dates: Vec<String> = page.notes.iter().map(|n| n.date.clone()).collect();
                        dates.into_iter().max()
                    } else {
                        None
                    };
                    
                    stats.push(CountryStatsItem {
                        slug,
                        note_count,
                        last_updated,
                    });
                }
            }
            
            Ok(stats)
        }
        None => Err("No vault opened".to_string()),
    }
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")] 
pub struct CountryStatsItem {
    pub slug: String,
    pub note_count: usize,
    pub last_updated: Option<String>,
}