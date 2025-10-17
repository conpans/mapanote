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
    let mut lines = content.lines();

    if lines.next()? != "---" {
        return None;
    }

    let mut id = String::new();
    let mut title = String::new();
    let mut date = String::new();
    let mut tags = Vec::new();
    let mut topic_id = None; // ← ADD
    let mut country_targets = Vec::new(); // ← ADD

    for line in lines.by_ref() {
        if line == "---" {
            break;
        }

        if let Some(value) = line.strip_prefix("id: ") {
            id = value.to_string();
        } else if let Some(value) = line.strip_prefix("title: ") {
            title = value.to_string();
        } else if let Some(value) = line.strip_prefix("date: ") {
            date = value.to_string();
        } else if let Some(value) = line.strip_prefix("tags: ") {
            let tags_str = value.trim_matches(|c| c == '[' || c == ']');
            tags = tags_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        } else if let Some(value) = line.strip_prefix("topic_id: ") {
            // ← ADD
            topic_id = Some(value.to_string());
        } else if let Some(value) = line.strip_prefix("country_targets: ") {
            // ← ADD
            let targets_str = value.trim_matches(|c| c == '[' || c == ']');
            country_targets = targets_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
    }

    let content: String = lines.collect::<Vec<_>>().join("\n").trim().to_string();

    Some(Note {
        id,
        title,
        content,
        date,
        tags,
        topic_id,        // ← ADD
        country_targets, // ← ADD
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
        id: id.clone(), // ← FIXED: use 'id' not 'note_id'
        title: title.clone(),
        content: content.clone(),
        date: date.clone(),
        tags: tags.clone(),
        topic_id: None,
        country_targets: vec![country_slug.clone()], // ← FIXED: use 'country_slug' not 'slug'
    };

    // Lazy-create country folder
    let country_dir = vault_root.join("notes").join(&country_slug);
    fs::create_dir_all(&country_dir)
        .map_err(|e| format!("Failed to create country directory: {}", e))?;

    // Write note file
    let note_filename = format!("{}.md", id);
    let note_path = country_dir.join(&note_filename);

    let note_content = format!(
        "---\nid: {}\ndate: {}\ntitle: {}\ntags: {:?}\ntopic_id: \ncountry_targets: [{}]\n---\n\n{}",
        id, date, title, tags, country_slug, content
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

#[derive(Serialize)]
pub struct RecentActivity {
    pub note_id: String,
    pub note_title: String,
    pub note_date: String,
    pub country_slug: String,
    pub country_name: String,
    pub source_type: String, // "country" or "topic"
    pub topic_name: Option<String>,
    pub topic_color: Option<String>,
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

#[tauri::command]
pub fn get_all_countries_with_combined_counts(
    state: State<AppState>,
) -> Result<Vec<mapanote_lib::models::CountryWithStats>, String> {
    let reader = state.vault_reader.lock().unwrap();
    let vault_path = reader.as_ref().ok_or("No vault opened")?;

    let vault_root = PathBuf::from(vault_path);
    let manifest_path = vault_root.join("vault.json");

    let manifest_str = fs::read_to_string(&manifest_path)
        .map_err(|e| format!("Failed to read manifest: {}", e))?;

    let manifest: VaultManifest = serde_json::from_str(&manifest_str)
        .map_err(|e| format!("Failed to parse manifest: {}", e))?;

    // Load topics manifest to count topic notes
    let topics_manifest = crate::commands::topics::load_topics_manifest(vault_path)
        .unwrap_or_else(|_| mapanote_lib::models::TopicsManifest::new());

    // Build a set of ALL country slugs that have either:
    // 1. Country notes (in vault.json)
    // 2. Topic notes targeting them
    let mut all_country_slugs = std::collections::HashSet::new();

    // Add countries from vault.json
    for slug in manifest.countries.keys() {
        all_country_slugs.insert(slug.clone());
    }

    // Add countries targeted by topic notes
    for topic in &topics_manifest.topics {
        let topic_notes_dir = vault_root.join("topics").join(&topic.id);

        if topic_notes_dir.exists() {
            if let Ok(entries) = fs::read_dir(&topic_notes_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("md") {
                        if let Ok(content) = fs::read_to_string(&path) {
                            if let Some(note) = parse_note(&content) {
                                // Add all country targets from this note
                                for target in note.country_targets {
                                    all_country_slugs.insert(target);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let mut countries: Vec<mapanote_lib::models::CountryWithStats> = Vec::new();

    // Process each country slug
    for slug in all_country_slugs {
        // Get base metadata
        let metadata = match crate::get_country_metadata(slug.clone()) {
            Ok(meta) => meta,
            Err(_) => continue, // Skip if metadata not found
        };

        // Get country notes count (from vault.json)
        let country_note_count = manifest
            .countries
            .get(&slug)
            .map(|stats| stats.note_count)
            .unwrap_or(0);

        // Get last_updated and tags from vault.json (if exists)
        let (last_updated, tags) = manifest
            .countries
            .get(&slug)
            .map(|stats| (stats.last_updated.clone(), stats.tags.clone()))
            .unwrap_or((None, Vec::new()));

        // Count topic notes targeting this country
        let mut topic_note_count = 0;

        for topic in &topics_manifest.topics {
            let topic_notes_dir = vault_root.join("topics").join(&topic.id);

            if topic_notes_dir.exists() {
                if let Ok(entries) = fs::read_dir(&topic_notes_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().and_then(|s| s.to_str()) == Some("md") {
                            if let Ok(content) = fs::read_to_string(&path) {
                                if let Some(note) = parse_note(&content) {
                                    // Only count if this country is in the targets
                                    if note.country_targets.contains(&slug) {
                                        topic_note_count += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Calculate total
        let total_note_count = country_note_count + topic_note_count;

        // Only include countries that have at least one note (country or topic)
        if total_note_count > 0 {
            countries.push(mapanote_lib::models::CountryWithStats {
                slug: metadata.slug,
                name: metadata.name,
                iso2: metadata.iso2,
                iso3: metadata.iso3,
                summary: metadata.summary,
                region: metadata.region,
                subregion: metadata.subregion,
                note_count: total_note_count,
                last_updated,
                tags,
            });
        }
    }

    countries.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(countries)
}

// Add this command at the end of vault.rs
#[tauri::command]
pub fn get_recent_activity(
    // ← Make sure it has 'pub'
    limit: usize,
    state: State<AppState>,
) -> Result<Vec<RecentActivity>, String> {
    let reader = state.vault_reader.lock().unwrap();
    let vault_path = reader.as_ref().ok_or("No vault opened")?;

    let vault_root = PathBuf::from(vault_path);
    let mut activities: Vec<RecentActivity> = Vec::new();

    // 1. Collect all country notes
    let notes_dir = vault_root.join("notes");
    if notes_dir.exists() {
        for country_entry in fs::read_dir(&notes_dir)
            .map_err(|e| format!("Failed to read notes directory: {}", e))?
        {
            let country_entry =
                country_entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let country_path = country_entry.path();

            if !country_path.is_dir() {
                continue;
            }

            let country_slug = country_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            // Get country metadata
            let country_metadata = match crate::get_country_metadata(country_slug.clone()) {
                Ok(meta) => meta,
                Err(_) => continue,
            };

            // Read notes in this country
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
                    activities.push(RecentActivity {
                        note_id: note.id,
                        note_title: note.title,
                        note_date: note.date,
                        country_slug: country_slug.clone(),
                        country_name: country_metadata.name.clone(),
                        source_type: "country".to_string(),
                        topic_name: None,
                        topic_color: None,
                    });
                }
            }
        }
    }

    // 2. Collect all topic notes
    let topics_manifest = crate::commands::topics::load_topics_manifest(vault_path)
        .unwrap_or_else(|_| mapanote_lib::models::TopicsManifest::new());

    for topic in &topics_manifest.topics {
        let topic_notes_dir = vault_root.join("topics").join(&topic.id);

        if !topic_notes_dir.exists() {
            continue;
        }

        for note_entry in fs::read_dir(&topic_notes_dir)
            .map_err(|e| format!("Failed to read topic notes directory: {}", e))?
        {
            let note_entry = note_entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let note_path = note_entry.path();

            if note_path.extension().and_then(|s| s.to_str()) != Some("md") {
                continue;
            }

            let content = fs::read_to_string(&note_path)
                .map_err(|e| format!("Failed to read note: {}", e))?;

            if let Some(note) = parse_note(&content) {
                // Get first country target as the primary country
                let primary_country_slug = note
                    .country_targets
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "unknown".to_string());

                let country_name = match crate::get_country_metadata(primary_country_slug.clone()) {
                    Ok(meta) => meta.name,
                    Err(_) => primary_country_slug.clone(),
                };

                activities.push(RecentActivity {
                    note_id: note.id,
                    note_title: note.title,
                    note_date: note.date,
                    country_slug: primary_country_slug,
                    country_name,
                    source_type: "topic".to_string(),
                    topic_name: Some(topic.title.clone()),
                    topic_color: topic.color.clone(),
                });
            }
        }
    }

    // Sort by date (newest first)
    activities.sort_by(|a, b| b.note_date.cmp(&a.note_date));

    // Limit results
    activities.truncate(limit);

    Ok(activities)
}
