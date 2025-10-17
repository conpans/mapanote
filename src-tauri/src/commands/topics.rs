use chrono::Utc;
use mapanote_lib::models::{
    Note, NoteWithSource, Topic, TopicCountryRelation, TopicWithCountries, TopicsManifest,
};
use mapanote_lib::AppState;
use std::fs;
use std::path::PathBuf;
use tauri::State;
use ulid::Ulid;

fn get_topics_path(vault_path: &str) -> PathBuf {
    PathBuf::from(vault_path).join("topics.json")
}

/// Load topics manifest, creating it if it doesn't exist
pub fn load_topics_manifest(vault_path: &str) -> Result<TopicsManifest, String> {
    let topics_path = get_topics_path(vault_path);

    if !topics_path.exists() {
        // Create new empty manifest
        let manifest = TopicsManifest::new();
        let json = serde_json::to_string_pretty(&manifest)
            .map_err(|e| format!("Failed to serialize manifest: {}", e))?;
        fs::write(&topics_path, json).map_err(|e| format!("Failed to write topics.json: {}", e))?;
        return Ok(manifest);
    }

    let content = fs::read_to_string(&topics_path)
        .map_err(|e| format!("Failed to read topics.json: {}", e))?;

    serde_json::from_str(&content).map_err(|e| format!("Failed to parse topics.json: {}", e))
}

/// Save topics manifest
fn save_topics_manifest(vault_path: &str, manifest: &TopicsManifest) -> Result<(), String> {
    let topics_path = get_topics_path(vault_path);
    let json = serde_json::to_string_pretty(manifest)
        .map_err(|e| format!("Failed to serialize manifest: {}", e))?;
    fs::write(&topics_path, json).map_err(|e| format!("Failed to write topics.json: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn get_all_topics(state: State<AppState>) -> Result<Vec<TopicWithCountries>, String> {
    let reader = state.vault_reader.lock().unwrap();
    let vault_path = reader.as_ref().ok_or("No vault opened")?;

    let manifest = load_topics_manifest(vault_path)?;

    // Convert to TopicWithCountries
    let topics: Vec<TopicWithCountries> = manifest
        .topics
        .into_iter()
        .map(|topic| {
            // Find all countries related to this topic
            let countries: Vec<String> = manifest
                .relations
                .iter()
                .filter(|r| r.topic_id == topic.id)
                .map(|r| r.country_slug.clone())
                .collect();

            // Calculate total note count
            let note_count: usize = manifest
                .relations
                .iter()
                .filter(|r| r.topic_id == topic.id)
                .map(|r| r.note_count)
                .sum();

            TopicWithCountries {
                topic,
                countries,
                note_count,
            }
        })
        .collect();

    Ok(topics)
}

#[tauri::command]
pub fn get_topic(topic_id: String, state: State<AppState>) -> Result<TopicWithCountries, String> {
    let reader = state.vault_reader.lock().unwrap();
    let vault_path = reader.as_ref().ok_or("No vault opened")?;

    let manifest = load_topics_manifest(vault_path)?;

    let topic = manifest
        .topics
        .iter()
        .find(|t| t.id == topic_id)
        .ok_or_else(|| format!("Topic {} not found", topic_id))?;

    // Find all countries related to this topic
    let countries: Vec<String> = manifest
        .relations
        .iter()
        .filter(|r| r.topic_id == topic_id)
        .map(|r| r.country_slug.clone())
        .collect();

    // Calculate total note count
    let note_count: usize = manifest
        .relations
        .iter()
        .filter(|r| r.topic_id == topic_id)
        .map(|r| r.note_count)
        .sum();

    Ok(TopicWithCountries {
        topic: topic.clone(),
        countries,
        note_count,
    })
}

#[tauri::command]
pub fn create_topic(
    title: String,
    summary: Option<String>,
    color: Option<String>,
    country_slugs: Vec<String>,
    state: State<AppState>,
) -> Result<Topic, String> {
    let writer = state.vault_writer.lock().unwrap();
    let vault_path = writer.as_ref().ok_or("No vault opened")?;

    let mut manifest = load_topics_manifest(vault_path)?;

    // Generate topic ID
    let id = ulid::Ulid::new().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let topic = Topic {
        id: id.clone(),
        title,
        summary,
        color,
        pinned: false,
        created_at: now.clone(),
        updated_at: now,
    };

    manifest.topics.push(topic.clone());

    // Create relations for each country
    for country_slug in country_slugs {
        manifest.relations.push(TopicCountryRelation {
            topic_id: id.clone(),
            country_slug,
            note_count: 0,
            last_updated: None,
        });
    }

    save_topics_manifest(vault_path, &manifest)?;

    Ok(topic)
}

#[tauri::command]
pub fn update_topic(
    topic_id: String,
    title: String,
    summary: Option<String>,
    color: Option<String>,
    pinned: bool,
    state: State<AppState>,
) -> Result<(), String> {
    let writer = state.vault_writer.lock().unwrap();
    let vault_path = writer.as_ref().ok_or("No vault opened")?;

    let mut manifest = load_topics_manifest(vault_path)?;

    let topic = manifest
        .topics
        .iter_mut()
        .find(|t| t.id == topic_id)
        .ok_or_else(|| format!("Topic {} not found", topic_id))?;

    topic.title = title;
    topic.summary = summary;
    topic.color = color;
    topic.pinned = pinned;
    topic.updated_at = chrono::Utc::now().to_rfc3339();

    save_topics_manifest(vault_path, &manifest)?;

    Ok(())
}

#[tauri::command]
pub fn delete_topic(topic_id: String, state: State<AppState>) -> Result<(), String> {
    let writer = state.vault_writer.lock().unwrap();
    let vault_path = writer.as_ref().ok_or("No vault opened")?;

    let mut manifest = load_topics_manifest(vault_path)?;

    // Remove topic
    manifest.topics.retain(|t| t.id != topic_id);

    // Remove all relations
    manifest.relations.retain(|r| r.topic_id != topic_id);

    save_topics_manifest(vault_path, &manifest)?;

    Ok(())
}

#[tauri::command]
pub fn add_country_to_topic(
    topic_id: String,
    country_slug: String,
    state: State<AppState>,
) -> Result<(), String> {
    let writer = state.vault_writer.lock().unwrap();
    let vault_path = writer.as_ref().ok_or("No vault opened")?;

    let mut manifest = load_topics_manifest(vault_path)?;

    // Check if relation already exists
    let exists = manifest
        .relations
        .iter()
        .any(|r| r.topic_id == topic_id && r.country_slug == country_slug);

    if exists {
        return Err("Country already in topic".to_string());
    }

    manifest.relations.push(TopicCountryRelation {
        topic_id,
        country_slug,
        note_count: 0,
        last_updated: None,
    });

    save_topics_manifest(vault_path, &manifest)?;

    Ok(())
}

#[tauri::command]
pub fn remove_country_from_topic(
    topic_id: String,
    country_slug: String,
    state: State<AppState>,
) -> Result<(), String> {
    let writer = state.vault_writer.lock().unwrap();
    let vault_path = writer.as_ref().ok_or("No vault opened")?;

    let mut manifest = load_topics_manifest(vault_path)?;

    manifest
        .relations
        .retain(|r| !(r.topic_id == topic_id && r.country_slug == country_slug));

    save_topics_manifest(vault_path, &manifest)?;

    Ok(())
}

#[tauri::command]
pub fn get_topics_for_country(
    country_slug: String,
    state: State<AppState>,
) -> Result<Vec<Topic>, String> {
    let reader = state.vault_reader.lock().unwrap();
    let vault_path = reader.as_ref().ok_or("No vault opened")?;

    let manifest = load_topics_manifest(vault_path)?;

    // Find all topic IDs for this country
    let topic_ids: Vec<String> = manifest
        .relations
        .iter()
        .filter(|r| r.country_slug == country_slug)
        .map(|r| r.topic_id.clone())
        .collect();

    // Get the full topic objects
    let topics: Vec<Topic> = manifest
        .topics
        .into_iter()
        .filter(|t| topic_ids.contains(&t.id))
        .collect();

    Ok(topics)
}

/// Add a note to a topic
#[tauri::command]
pub fn add_topic_note(
    topic_id: String,
    title: String,
    content: String,
    tags: Vec<String>,
    country_targets: Vec<String>, // Which countries this note is about
    state: State<AppState>,
) -> Result<Note, String> {
    let writer = state.vault_writer.lock().unwrap();
    let vault_path = writer.as_ref().ok_or("No vault opened")?;

    let vault_root = PathBuf::from(vault_path);
    let topic_notes_dir = vault_root.join("topics").join(&topic_id);

    // Create topics directory if it doesn't exist
    fs::create_dir_all(&topic_notes_dir)
        .map_err(|e| format!("Failed to create topic notes directory: {}", e))?;

    // Generate note ID and filename
    let note_id = Ulid::new().to_string();
    let filename = format!("{}.md", note_id);
    let note_path = topic_notes_dir.join(&filename);

    // Get current date
    let now = Utc::now();
    let date = now.format("%Y-%m-%d").to_string();

    // Create note
    let note = Note {
        id: note_id.clone(),
        title: title.clone(),
        content: content.clone(),
        date: date.clone(),
        tags: tags.clone(),
        topic_id: Some(topic_id.clone()),
        country_targets: country_targets.clone(),
    };

    // Format note content with country_targets
    let note_content = format!(
        "---\nid: {}\ntitle: {}\ndate: {}\ntags: [{}]\ntopic_id: {}\ncountry_targets: [{}]\n---\n\n{}",
        note_id,
        title,
        date,
        tags.join(", "),
        topic_id,
        country_targets.join(", "),
        content
    );

    // Write note to file
    fs::write(&note_path, note_content).map_err(|e| format!("Failed to write note: {}", e))?;

    Ok(note)
}

/// Get all notes for a topic (regardless of country targets)
#[tauri::command]
pub fn get_topic_notes(topic_id: String, state: State<AppState>) -> Result<Vec<Note>, String> {
    let reader = state.vault_reader.lock().unwrap();
    let vault_path = reader.as_ref().ok_or("No vault opened")?;

    let vault_root = PathBuf::from(vault_path);
    let topic_notes_dir = vault_root.join("topics").join(&topic_id);

    if !topic_notes_dir.exists() {
        return Ok(Vec::new());
    }

    let mut notes = Vec::new();

    for entry in fs::read_dir(&topic_notes_dir)
        .map_err(|e| format!("Failed to read topic notes directory: {}", e))?
    {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("md") {
            let content =
                fs::read_to_string(&path).map_err(|e| format!("Failed to read note: {}", e))?;

            if let Some(note) = parse_note(&content) {
                notes.push(note);
            }
        }
    }

    // Sort by date (newest first)
    notes.sort_by(|a, b| b.date.cmp(&a.date));

    Ok(notes)
}

/// Update a topic note
#[tauri::command]
pub fn update_topic_note(
    topic_id: String,
    note_id: String,
    title: String,
    content: String,
    tags: Vec<String>,
    country_targets: Vec<String>,
    state: State<AppState>,
) -> Result<(), String> {
    let writer = state.vault_writer.lock().unwrap();
    let vault_path = writer.as_ref().ok_or("No vault opened")?;

    let vault_root = PathBuf::from(vault_path);
    let note_path = vault_root
        .join("topics")
        .join(&topic_id)
        .join(format!("{}.md", note_id));

    if !note_path.exists() {
        return Err(format!("Note {} not found", note_id));
    }

    // Read existing note to preserve date
    let existing_content =
        fs::read_to_string(&note_path).map_err(|e| format!("Failed to read note: {}", e))?;

    let existing_note = parse_note(&existing_content).ok_or("Failed to parse existing note")?;

    // Format updated note content
    let note_content = format!(
        "---\nid: {}\ntitle: {}\ndate: {}\ntags: [{}]\ntopic_id: {}\ncountry_targets: [{}]\n---\n\n{}",
        note_id,
        title,
        existing_note.date,
        tags.join(", "),
        topic_id,
        country_targets.join(", "),
        content
    );

    // Write updated note
    fs::write(&note_path, note_content).map_err(|e| format!("Failed to write note: {}", e))?;

    Ok(())
}

/// Delete a topic note
#[tauri::command]
pub fn delete_topic_note(
    topic_id: String,
    note_id: String,
    state: State<AppState>,
) -> Result<(), String> {
    let writer = state.vault_writer.lock().unwrap();
    let vault_path = writer.as_ref().ok_or("No vault opened")?;

    let vault_root = PathBuf::from(vault_path);
    let note_path = vault_root
        .join("topics")
        .join(&topic_id)
        .join(format!("{}.md", note_id));

    if note_path.exists() {
        fs::remove_file(&note_path).map_err(|e| format!("Failed to delete note: {}", e))?;
    }

    Ok(())
}

/// Get all notes for a country (country notes + topic notes that target this country)
#[tauri::command]
pub fn get_country_notes_with_topics(
    slug: String,
    state: State<AppState>,
) -> Result<Vec<NoteWithSource>, String> {
    let reader = state.vault_reader.lock().unwrap();
    let vault_path = reader.as_ref().ok_or("No vault opened")?;

    let mut all_notes = Vec::new();

    // 1. Get regular country notes
    let vault_root = PathBuf::from(vault_path);
    let country_notes_dir = vault_root.join("notes").join(&slug);

    if country_notes_dir.exists() {
        for entry in fs::read_dir(&country_notes_dir)
            .map_err(|e| format!("Failed to read notes directory: {}", e))?
        {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                let content =
                    fs::read_to_string(&path).map_err(|e| format!("Failed to read note: {}", e))?;

                if let Some(note) = parse_note(&content) {
                    all_notes.push(NoteWithSource {
                        note,
                        source_type: "country".to_string(),
                        source_name: slug.clone(),
                        topic_color: None,
                    });
                }
            }
        }
    }

    // 2. Get topic notes that TARGET this country
    let manifest = load_topics_manifest(vault_path)?;
    let topic_ids: Vec<String> = manifest
        .relations
        .iter()
        .filter(|r| r.country_slug == slug)
        .map(|r| r.topic_id.clone())
        .collect();

    for topic_id in topic_ids {
        let topic_notes_dir = vault_root.join("topics").join(&topic_id);

        if !topic_notes_dir.exists() {
            continue;
        }

        // Find topic details
        let topic = manifest.topics.iter().find(|t| t.id == topic_id);
        let topic_name = topic
            .as_ref()
            .map(|t| t.title.clone())
            .unwrap_or(topic_id.clone());
        let topic_color = topic.as_ref().and_then(|t| t.color.clone());

        for entry in fs::read_dir(&topic_notes_dir)
            .map_err(|e| format!("Failed to read topic notes directory: {}", e))?
        {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                let content =
                    fs::read_to_string(&path).map_err(|e| format!("Failed to read note: {}", e))?;

                if let Some(note) = parse_note(&content) {
                    // CRITICAL: Only include if this country is in country_targets
                    if note.country_targets.contains(&slug) {
                        all_notes.push(NoteWithSource {
                            note,
                            source_type: "topic".to_string(),
                            source_name: topic_name.clone(),
                            topic_color: topic_color.clone(),
                        });
                    }
                }
            }
        }
    }

    // Sort by date (newest first)
    all_notes.sort_by(|a, b| b.note.date.cmp(&a.note.date));

    Ok(all_notes)
}

// Helper function to parse note with country_targets
fn parse_note(content: &str) -> Option<Note> {
    let mut lines = content.lines();

    if lines.next()? != "---" {
        return None;
    }

    let mut id = String::new();
    let mut title = String::new();
    let mut date = String::new();
    let mut tags = Vec::new();
    let mut topic_id = None;
    let mut country_targets = Vec::new();

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
            topic_id = Some(value.to_string());
        } else if let Some(value) = line.strip_prefix("country_targets: ") {
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
        topic_id,
        country_targets,
    })
}
