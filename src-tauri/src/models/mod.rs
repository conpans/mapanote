use serde::{Deserialize, Serialize};

/// Represents a country entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Country {
    pub slug: String,
    pub title: String,
    pub region: String,
    pub summary: String,
    pub aliases: Vec<String>,
    pub updated_at: String,
}

/// Represents a note/card
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub date: String,
    pub tags: Vec<String>,
    pub text: String,
    pub also: Vec<String>, // Cross-references
    pub visibility: Visibility,
}

/// Visibility level for notes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    Private,
    Internal,
    Publishable,
}

impl Default for Visibility {
    fn default() -> Self {
        Self::Internal
    }
}

/// Vault configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultConfig {
    pub schema_version: u32,
    pub created_at: String,
    pub app_version: Option<String>,
}

/// Parsed country page (frontmatter + notes)
#[derive(Debug, Clone)]
pub struct CountryPage {
    pub country: Country,
    pub notes: Vec<Note>,
    pub raw_content: String, // Markdown body
}
