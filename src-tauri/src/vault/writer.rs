use super::get_country_metadata;
use super::reader::VaultReader;
use crate::models::{AddNoteRequest, Note, UpdateNoteRequest, Visibility};
use anyhow::{Context, Result};
use chrono::Utc;
use std::fs;
use std::path::PathBuf;

pub struct VaultWriter {
    vault_path: PathBuf,
}

impl VaultWriter {
    pub fn new(vault_path: PathBuf) -> Self {
        Self { vault_path }
    }

    /// Add a new note to a country
    pub fn add_note(&self, request: AddNoteRequest) -> Result<Note> {
        self.ensure_country(&request.country_slug)?;

        let country_path = self
            .vault_path
            .join("countries")
            .join(&request.country_slug)
            .join("index.md");

        // Read existing file
        let mut contents =
            fs::read_to_string(&country_path).context("Failed to read country file")?;

        // Generate new note ID
        let note_id = ulid::Ulid::new().to_string();

        // Get today's date if not provided
        let date = Utc::now().format("%Y-%m-%d").to_string();

        // Build note header
        let mut header_parts = vec![date.clone()];

        // Add tags
        if !request.tags.is_empty() {
            header_parts.push(request.tags.join(", "));
        }

        // Add metadata
        let mut metadata_parts = Vec::new();

        if !request.also.is_empty() {
            metadata_parts.push(format!("also:{}", request.also.join(",")));
        }

        metadata_parts.push(match request.visibility {
            Visibility::Private => "private".to_string(),
            Visibility::Internal => "internal".to_string(),
            Visibility::Publishable => "publishable".to_string(),
        });

        if request.pinned {
            metadata_parts.push("pinned".to_string());
        }

        if !metadata_parts.is_empty() {
            header_parts.push(metadata_parts.join(" · "));
        }

        // Build note markdown
        let note_md = format!(
            "\n### {} · {}\n[id:{}]\n\n{}\n",
            date,
            header_parts[1..].join(" · "),
            note_id,
            request.text.trim()
        );

        // Append to file
        contents.push_str(&note_md);

        // Write back
        fs::write(&country_path, contents).context("Failed to write country file")?;

        // Return the created note
        Ok(Note {
            id: note_id,
            date,
            tags: request.tags,
            text: request.text,
            also: request.also,
            visibility: request.visibility,
            pinned: request.pinned,
        })
    }

    /// Update an existing note
    pub fn update_note(&self, request: UpdateNoteRequest) -> Result<()> {
        let country_path = self
            .vault_path
            .join("countries")
            .join(&request.country_slug)
            .join("index.md");

        // Read file
        let contents = fs::read_to_string(&country_path).context("Failed to read country file")?;

        // Parse to find the note
        let reader = VaultReader::new(self.vault_path.clone());
        let page = reader.read_country(&request.country_slug)?;

        let note = page
            .notes
            .iter()
            .find(|n| n.id == request.note_id)
            .context("Note not found")?;

        // Build new note text
        let mut header_parts = vec![note.date.clone()];

        if !request.tags.is_empty() {
            header_parts.push(request.tags.join(", "));
        }

        let mut metadata_parts = Vec::new();

        if !request.also.is_empty() {
            metadata_parts.push(format!("also:{}", request.also.join(",")));
        }

        metadata_parts.push(match request.visibility {
            Visibility::Private => "private".to_string(),
            Visibility::Internal => "internal".to_string(),
            Visibility::Publishable => "publishable".to_string(),
        });

        if request.pinned {
            metadata_parts.push("pinned".to_string());
        }

        if !metadata_parts.is_empty() {
            header_parts.push(metadata_parts.join(" · "));
        }

        let new_note = format!(
            "### {} · {}\n[id:{}]\n\n{}",
            note.date,
            header_parts[1..].join(" · "),
            request.note_id,
            request.text.trim()
        );

        // Replace old note with new note
        // Find the note by ID marker
        let id_marker = format!("[id:{}]", request.note_id);

        if let Some(start_pos) = contents.find(&id_marker) {
            // Find start of this note (go back to find ###)
            let before = &contents[..start_pos];
            let note_start = before.rfind("###").unwrap_or(0);

            // Find end of this note (next ### or end of file)
            let after = &contents[start_pos..];
            let next_note = after
                .find("\n###")
                .map(|p| start_pos + p)
                .unwrap_or(contents.len());

            // Replace
            let mut new_contents = String::new();
            new_contents.push_str(&contents[..note_start]);
            new_contents.push_str(&new_note);
            new_contents.push('\n');
            if next_note < contents.len() {
                new_contents.push_str(&contents[next_note..]);
            }

            fs::write(&country_path, new_contents).context("Failed to write country file")?;
        } else {
            anyhow::bail!("Note ID not found in file");
        }

        Ok(())
    }

    /// Delete a note
    pub fn delete_note(&self, country_slug: &str, note_id: &str) -> Result<()> {
        let country_path = self
            .vault_path
            .join("countries")
            .join(country_slug)
            .join("index.md");

        let contents = fs::read_to_string(&country_path)?;
        let id_marker = format!("[id:{}]", note_id);

        if let Some(start_pos) = contents.find(&id_marker) {
            let before = &contents[..start_pos];
            let note_start = before.rfind("###").unwrap_or(0);

            let after = &contents[start_pos..];
            let next_note = after
                .find("\n###")
                .map(|p| start_pos + p)
                .unwrap_or(contents.len());

            let mut new_contents = String::new();
            new_contents.push_str(&contents[..note_start]);
            if next_note < contents.len() {
                new_contents.push_str(&contents[next_note + 1..]); // +1 to skip the newline
            }

            fs::write(&country_path, new_contents)?;
        }

        Ok(())
    }

    pub fn ensure_country(&self, slug: &str) -> Result<bool> {
        let country_path = self
            .vault_path
            .join("countries")
            .join(slug)
            .join("index.md");

        // If file exists, nothing to do
        if country_path.exists() {
            return Ok(false);
        }

        // Get metadata from embedded data
        let metadata = get_country_metadata(slug)
            .ok_or_else(|| anyhow::anyhow!("Unknown country: {}", slug))?;

        // Create directory
        fs::create_dir_all(country_path.parent().unwrap())?;

        // Generate frontmatter
        let today = Utc::now().format("%Y-%m-%d").to_string();
        let aliases_yaml = if metadata.aliases.is_empty() {
            "[]".to_string()
        } else {
            format!(
                "[{}]",
                metadata
                    .aliases
                    .iter()
                    .map(|a| format!("\"{}\"", a))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };

        let content = format!(
            r#"---
    title: {}
    slug: {}
    region: {}
    summary: {}
    aliases: {}
    updated_at: {}
    ---

    ## Overview

    {}

    ## Notes

    <!-- Add your first note below -->
    "#,
            metadata.name,
            metadata.slug,
            metadata.subregion,
            metadata.summary,
            aliases_yaml,
            today,
            metadata.summary
        );

        // Write file
        fs::write(&country_path, content)?;

        eprintln!("Created country stub: {}", slug);
        Ok(true)
    }
}
