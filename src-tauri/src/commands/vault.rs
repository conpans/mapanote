use mapanote_lib::models::{CountryStats, Note, VaultManifest};
use mapanote_lib::AppState;
use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use tauri::State;

#[tauri::command]
pub fn create_minimal_vault(destination: String, vault_name: String) -> Result<String, String> {
    println!("Creating minimal vault at: {}", destination);

    let dest_path = PathBuf::from(&destination);

    // Create directory structure
    fs::create_dir_all(&dest_path)
        .map_err(|e| format!("Failed to create vault directory: {}", e))?;

    fs::create_dir_all(dest_path.join("notes"))
        .map_err(|e| format!("Failed to create notes directory: {}", e))?;

    fs::create_dir_all(dest_path.join(".mapanote"))
        .map_err(|e| format!("Failed to create config directory: {}", e))?;

    // Create empty manifest
    let manifest = VaultManifest::new();
    let manifest_path = dest_path.join("vault.json");
    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&manifest)
            .map_err(|e| format!("Failed to serialize manifest: {}", e))?,
    )
    .map_err(|e| format!("Failed to write manifest: {}", e))?;

    // Create config
    let config = serde_json::json!({
        "name": vault_name,
        "version": "1.0",
        "created": chrono::Utc::now().to_rfc3339(),
    });

    fs::write(
        dest_path.join(".mapanote/config.json"),
        serde_json::to_string_pretty(&config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?,
    )
    .map_err(|e| format!("Failed to write config: {}", e))?;

    // Create README
    let readme = format!(
        "# {}\n\nCreated: {}\n\n## Structure\n\n\
         - `vault.json` - Manifest tracking all notes\n\
         - `notes/` - Country folders (created when you add notes)\n\
         - `.mapanote/` - App configuration\n",
        vault_name,
        chrono::Utc::now().format("%Y-%m-%d")
    );

    fs::write(dest_path.join("README.md"), readme)
        .map_err(|e| format!("Failed to write README: {}", e))?;

    println!("✅ Minimal vault created successfully!");
    Ok(destination)
}

#[tauri::command]
pub fn open_vault(path: String, state: State<AppState>) -> Result<String, String> {
    let vault_path = PathBuf::from(&path);

    // Verify it's a valid vault
    let manifest_path = vault_path.join("vault.json");
    if !manifest_path.exists() {
        return Err("Not a valid Mapanote vault (vault.json not found)".to_string());
    }

    // Load and validate manifest
    let manifest_str = fs::read_to_string(&manifest_path)
        .map_err(|e| format!("Failed to read manifest: {}", e))?;

    let _manifest: VaultManifest = serde_json::from_str(&manifest_str)
        .map_err(|e| format!("Invalid manifest format: {}", e))?;

    // Store vault path
    let mut reader = state.vault_reader.lock().unwrap();
    *reader = Some(path.clone());

    let mut writer = state.vault_writer.lock().unwrap();
    *writer = Some(path.clone());

    Ok(format!("Opened vault at {}", path))
}

#[tauri::command]
pub fn get_vault_manifest(state: State<AppState>) -> Result<VaultManifest, String> {
    let reader = state.vault_reader.lock().unwrap();
    let vault_path = reader.as_ref().ok_or("No vault opened")?;

    let manifest_path = PathBuf::from(vault_path).join("vault.json");
    let manifest_str = fs::read_to_string(&manifest_path)
        .map_err(|e| format!("Failed to read manifest: {}", e))?;

    serde_json::from_str(&manifest_str).map_err(|e| format!("Failed to parse manifest: {}", e))
}

#[tauri::command]
pub fn get_country_notes(slug: String, state: State<AppState>) -> Result<Vec<Note>, String> {
    let reader = state.vault_reader.lock().unwrap();
    let vault_path = reader.as_ref().ok_or("No vault opened")?;

    let country_dir = PathBuf::from(vault_path).join("notes").join(&slug);

    // If folder doesn't exist, return empty array (not an error)
    if !country_dir.exists() {
        return Ok(Vec::new());
    }

    let mut notes = Vec::new();

    for entry in fs::read_dir(&country_dir)
        .map_err(|e| format!("Failed to read country directory: {}", e))?
    {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("md") {
            let content =
                fs::read_to_string(&path).map_err(|e| format!("Failed to read note: {}", e))?;

            // Parse frontmatter and content
            if let Some(note) = parse_note(&content) {
                notes.push(note);
            }
        }
    }

    // Sort by date descending
    notes.sort_by(|a, b| b.date.cmp(&a.date));

    Ok(notes)
}

fn parse_note(content: &str) -> Option<Note> {
    // Simple frontmatter parser
    if !content.starts_with("---\n") {
        return None;
    }

    let parts: Vec<&str> = content.splitn(3, "---\n").collect();
    if parts.len() < 3 {
        return None;
    }

    let frontmatter = parts[1];
    let body = parts[2].trim();

    let mut id = String::new();
    let mut date = String::new();
    let mut title = String::new();
    let mut tags = Vec::new();

    for line in frontmatter.lines() {
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            let value = value.trim();

            match key {
                "id" => id = value.to_string(),
                "date" => date = value.to_string(),
                "title" => title = value.to_string(),
                "tags" => {
                    // Parse array: ["tag1", "tag2"]
                    tags = value
                        .trim_matches(|c| c == '[' || c == ']')
                        .split(',')
                        .map(|s| s.trim().trim_matches('"').to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                }
                _ => {}
            }
        }
    }

    Some(Note {
        id,
        date,
        title,
        content: body.to_string(),
        tags,
    })
}

#[tauri::command]
pub fn add_note(
    country_slug: String,
    title: String,
    content: String,
    tags: Vec<String>,
    state: State<AppState>,
) -> Result<Note, String> {
    let writer = state.vault_writer.lock().unwrap();
    let vault_path = writer.as_ref().ok_or("No vault opened")?;

    let vault_root = PathBuf::from(vault_path);

    // Generate note ID and date
    let id = ulid::Ulid::new().to_string();
    let date = chrono::Utc::now().format("%Y-%m-%d").to_string();

    let note = Note {
        id: id.clone(),
        date: date.clone(),
        title: title.clone(),
        content: content.clone(),
        tags: tags.clone(),
    };

    // Lazy-create country folder
    let country_dir = vault_root.join("notes").join(&country_slug);
    fs::create_dir_all(&country_dir)
        .map_err(|e| format!("Failed to create country directory: {}", e))?;

    // Write note file
    let note_filename = format!("{}.md", id);
    let note_path = country_dir.join(&note_filename);

    let note_content = format!(
        "---\nid: {}\ndate: {}\ntitle: {}\ntags: {:?}\n---\n\n{}",
        id, date, title, tags, content
    );

    fs::write(&note_path, note_content).map_err(|e| format!("Failed to write note: {}", e))?;

    // Update manifest
    let manifest_path = vault_root.join("vault.json");
    let manifest_str = fs::read_to_string(&manifest_path)
        .map_err(|e| format!("Failed to read manifest: {}", e))?;

    let mut manifest: VaultManifest = serde_json::from_str(&manifest_str)
        .map_err(|e| format!("Failed to parse manifest: {}", e))?;

    let stats = manifest
        .countries
        .entry(country_slug.clone())
        .or_insert(CountryStats {
            note_count: 0,
            last_updated: None,
            tags: Vec::new(),
        });

    stats.note_count += 1;
    stats.last_updated = Some(date.clone());

    // Add new tags
    for tag in &tags {
        if !stats.tags.contains(tag) {
            stats.tags.push(tag.clone());
        }
    }
    stats.tags.sort();

    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&manifest)
            .map_err(|e| format!("Failed to serialize manifest: {}", e))?,
    )
    .map_err(|e| format!("Failed to write manifest: {}", e))?;

    Ok(note)
}

#[tauri::command]
pub fn list_countries(state: State<AppState>) -> Result<Vec<String>, String> {
    let reader = state.vault_reader.lock().unwrap();
    let vault_path = reader.as_ref().ok_or("No vault opened")?;

    let manifest_path = PathBuf::from(vault_path).join("vault.json");
    let manifest_str = fs::read_to_string(&manifest_path)
        .map_err(|e| format!("Failed to read manifest: {}", e))?;

    let manifest: VaultManifest = serde_json::from_str(&manifest_str)
        .map_err(|e| format!("Failed to parse manifest: {}", e))?;

    let mut countries: Vec<String> = manifest.countries.keys().cloned().collect();
    countries.sort();

    Ok(countries)
}

#[derive(Serialize)]
pub struct CountryStatsWithSlug {
    // ← ADD pub
    slug: String,
    #[serde(rename = "noteCount")]
    note_count: usize,
    #[serde(rename = "lastUpdated")]
    last_updated: Option<String>,
    tags: Vec<String>,
}

#[tauri::command]
pub fn get_all_country_stats(state: State<AppState>) -> Result<Vec<CountryStatsWithSlug>, String> {
    let manifest = get_vault_manifest(state)?;

    let stats: Vec<CountryStatsWithSlug> = manifest
        .countries
        .into_iter()
        .map(|(slug, stats)| CountryStatsWithSlug {
            slug,
            note_count: stats.note_count,
            last_updated: stats.last_updated,
            tags: stats.tags,
        })
        .collect();

    Ok(stats)
}

#[tauri::command]
pub fn update_note(
    country_slug: String,
    note_id: String,
    title: String,
    content: String,
    tags: Vec<String>,
    state: State<AppState>,
) -> Result<(), String> {
    let writer = state.vault_writer.lock().unwrap();
    let vault_path = writer.as_ref().ok_or("No vault opened")?;

    let vault_root = PathBuf::from(vault_path);
    let note_path = vault_root
        .join("notes")
        .join(&country_slug)
        .join(format!("{}.md", note_id));

    // Check if note exists
    if !note_path.exists() {
        return Err(format!("Note {} not found", note_id));
    }

    // Read existing note to get the date
    let existing_content =
        fs::read_to_string(&note_path).map_err(|e| format!("Failed to read note: {}", e))?;

    let existing_date = if let Some(note) = parse_note(&existing_content) {
        note.date
    } else {
        chrono::Utc::now().format("%Y-%m-%d").to_string()
    };

    // Write updated note
    let note_content = format!(
        "---\nid: {}\ndate: {}\ntitle: {}\ntags: {:?}\n---\n\n{}",
        note_id, existing_date, title, tags, content
    );

    fs::write(&note_path, note_content).map_err(|e| format!("Failed to write note: {}", e))?;

    // Update manifest tags
    let manifest_path = vault_root.join("vault.json");
    let manifest_str = fs::read_to_string(&manifest_path)
        .map_err(|e| format!("Failed to read manifest: {}", e))?;

    let mut manifest: VaultManifest = serde_json::from_str(&manifest_str)
        .map_err(|e| format!("Failed to parse manifest: {}", e))?;

    if let Some(stats) = manifest.countries.get_mut(&country_slug) {
        // Update tags
        stats.tags = tags.clone();
        stats.tags.sort();
        stats.tags.dedup();
    }

    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&manifest)
            .map_err(|e| format!("Failed to serialize manifest: {}", e))?,
    )
    .map_err(|e| format!("Failed to write manifest: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn delete_note(
    country_slug: String,
    note_id: String,
    state: State<AppState>,
) -> Result<(), String> {
    let writer = state.vault_writer.lock().unwrap();
    let vault_path = writer.as_ref().ok_or("No vault opened")?;

    let vault_root = PathBuf::from(vault_path);
    let note_path = vault_root
        .join("notes")
        .join(&country_slug)
        .join(format!("{}.md", note_id));

    // Delete the note file
    if note_path.exists() {
        fs::remove_file(&note_path).map_err(|e| format!("Failed to delete note: {}", e))?;
    } else {
        return Err(format!("Note {} not found", note_id));
    }

    // Update manifest
    let manifest_path = vault_root.join("vault.json");
    let manifest_str = fs::read_to_string(&manifest_path)
        .map_err(|e| format!("Failed to read manifest: {}", e))?;

    let mut manifest: VaultManifest = serde_json::from_str(&manifest_str)
        .map_err(|e| format!("Failed to parse manifest: {}", e))?;

    if let Some(stats) = manifest.countries.get_mut(&country_slug) {
        if stats.note_count > 0 {
            stats.note_count -= 1;
        }

        // If no notes left, remove country from manifest
        if stats.note_count == 0 {
            manifest.countries.remove(&country_slug);
        }
    }

    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&manifest)
            .map_err(|e| format!("Failed to serialize manifest: {}", e))?,
    )
    .map_err(|e| format!("Failed to write manifest: {}", e))?;

    Ok(())
}

#[derive(Serialize)]
pub struct SearchResult {
    pub country_slug: String,
    pub country_name: String,
    pub note_id: String,
    pub note_title: String,
    pub note_date: String,
    pub snippet: String,
    pub tags: Vec<String>,
}

#[tauri::command]
pub fn search_notes(query: String, state: State<AppState>) -> Result<Vec<SearchResult>, String> {
    let reader = state.vault_reader.lock().unwrap();
    let vault_path = reader.as_ref().ok_or("No vault opened")?;

    let vault_root = PathBuf::from(vault_path);
    let notes_dir = vault_root.join("notes");

    if !notes_dir.exists() {
        return Ok(Vec::new());
    }

    let query_lower = query.to_lowercase();
    let mut results = Vec::new();

    // Iterate through country folders
    for country_entry in
        fs::read_dir(&notes_dir).map_err(|e| format!("Failed to read notes directory: {}", e))?
    {
        let country_entry = country_entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let country_path = country_entry.path();

        if !country_path.is_dir() {
            continue;
        }

        let country_slug = country_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        // Get country name from metadata
        let country_name = country_slug.clone(); // Fallback to slug

        // Iterate through notes in this country
        for note_entry in fs::read_dir(&country_path)
            .map_err(|e| format!("Failed to read country directory: {}", e))?
        {
            let note_entry = note_entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let note_path = note_entry.path();

            if note_path.extension().and_then(|s| s.to_str()) != Some("md") {
                continue;
            }

            let content = fs::read_to_string(&note_path)
                .map_err(|e| format!("Failed to read note: {}", e))?;

            if let Some(note) = parse_note(&content) {
                // Check if query matches title or content
                let title_lower = note.title.to_lowercase();
                let content_lower = note.content.to_lowercase();

                if title_lower.contains(&query_lower) || content_lower.contains(&query_lower) {
                    // Create snippet (first 150 chars of content)
                    let snippet = if note.content.len() > 150 {
                        format!("{}...", &note.content[..150])
                    } else {
                        note.content.clone()
                    };

                    results.push(SearchResult {
                        country_slug: country_slug.clone(),
                        country_name: country_name.clone(),
                        note_id: note.id,
                        note_title: note.title,
                        note_date: note.date,
                        snippet,
                        tags: note.tags,
                    });
                }
            }
        }
    }

    // Sort by date (newest first)
    results.sort_by(|a, b| b.note_date.cmp(&a.note_date));

    Ok(results)
}
