use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultManifest {
    pub version: String,
    pub created: String,
    pub countries: HashMap<String, CountryStats>,
}

impl VaultManifest {
    pub fn new() -> Self {
        Self {
            version: "1.0".to_string(),
            created: chrono::Utc::now().to_rfc3339(),
            countries: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountryStats {
    #[serde(rename = "noteCount")]
    pub note_count: usize,
    #[serde(rename = "lastUpdated")]
    pub last_updated: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountryMetadata {
    pub slug: String,
    pub name: String,
    pub iso2: String,
    pub iso3: String,
    pub summary: String,
    pub region: String,
    pub subregion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountryWithStats {
    pub slug: String,
    pub name: String,
    pub iso2: String,
    pub iso3: String,
    pub summary: String,
    pub region: String,
    pub subregion: String,
    // Stats fields
    pub note_count: usize,
    pub last_updated: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub title: String,
    pub content: String,
    pub date: String,
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic_id: Option<String>, // Which topic this belongs to (if any)
    #[serde(default)]
    pub country_targets: Vec<String>, // Which countries this note is about
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteWithSource {
    #[serde(flatten)]
    pub note: Note,
    pub source_type: String, // "country" or "topic"
    pub source_name: String, // country slug or topic title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic_color: Option<String>, // For visual distinction
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Topic {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    pub pinned: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicCountryRelation {
    pub topic_id: String,
    pub country_slug: String,
    pub note_count: usize,
    pub last_updated: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicsManifest {
    pub version: String,
    pub topics: Vec<Topic>,
    pub relations: Vec<TopicCountryRelation>,
}

impl TopicsManifest {
    pub fn new() -> Self {
        Self {
            version: "1.0".to_string(),
            topics: Vec::new(),
            relations: Vec::new(),
        }
    }
}

// Helper struct for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicWithCountries {
    #[serde(flatten)]
    pub topic: Topic,
    pub countries: Vec<String>,
    pub note_count: usize,
}
