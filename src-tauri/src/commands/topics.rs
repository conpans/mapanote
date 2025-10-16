use mapanote_lib::models::{Topic, TopicCountryRelation, TopicWithCountries, TopicsManifest};
use mapanote_lib::AppState;
use std::fs;
use std::path::PathBuf;
use tauri::State;

/// Get the path to topics.json
fn get_topics_path(vault_path: &str) -> PathBuf {
    PathBuf::from(vault_path).join("topics.json")
}

/// Load topics manifest, creating it if it doesn't exist
fn load_topics_manifest(vault_path: &str) -> Result<TopicsManifest, String> {
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
