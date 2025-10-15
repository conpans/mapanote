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
pub struct Note {
    pub id: String,
    pub date: String,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
}
