use super::frontmatter::{parse_country_frontmatter, split_frontmatter};
use super::notes::parse_notes;
use crate::models::{Country, CountryPage, VaultConfig};
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub struct VaultReader {
    vault_path: PathBuf,
}

impl VaultReader {
    /// Create a new vault reader
    pub fn new(vault_path: PathBuf) -> Self {
        Self { vault_path }
    }

    /// Load vault configuration
    pub fn load_config(&self) -> Result<VaultConfig> {
        let config_path = self.vault_path.join("vault.json");
        let contents = fs::read_to_string(&config_path).context("Failed to read vault.json")?;

        let config: VaultConfig =
            serde_json::from_str(&contents).context("Failed to parse vault.json")?;

        Ok(config)
    }

    /// List all country slugs
    pub fn list_countries(&self) -> Result<Vec<String>> {
        let countries_dir = self.vault_path.join("countries");

        if !countries_dir.exists() {
            return Ok(Vec::new());
        }

        let mut slugs = Vec::new();

        for entry in fs::read_dir(countries_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Some(slug) = entry.file_name().to_str() {
                    slugs.push(slug.to_string());
                }
            }
        }

        slugs.sort();
        Ok(slugs)
    }

    /// Read a country's full page (metadata + notes)
    pub fn read_country(&self, slug: &str) -> Result<CountryPage> {
        let country_path = self
            .vault_path
            .join("countries")
            .join(slug)
            .join("index.md");

        let contents = fs::read_to_string(&country_path)
            .context(format!("Failed to read country: {}", slug))?;

        self.parse_country_page(&contents, slug)
    }

    /// Parse country markdown into structured data
    fn parse_country_page(&self, contents: &str, slug: &str) -> Result<CountryPage> {
        // Split frontmatter and body
        let (frontmatter_opt, body) = split_frontmatter(contents)?;

        // Parse frontmatter into Country struct
        let country = if let Some(frontmatter) = frontmatter_opt {
            parse_country_frontmatter(&frontmatter, slug)?
        } else {
            // No frontmatter - create minimal country
            Country {
                slug: slug.to_string(),
                title: slug.to_uppercase(),
                region: "Unknown".to_string(),
                summary: String::new(),
                aliases: Vec::new(),
                updated_at: String::new(),
            }
        };

        // Parse notes from body
        let notes = parse_notes(&body)?;

        Ok(CountryPage {
            country,
            notes,
            raw_content: body,
        })
    }

    /// Get vault path
    pub fn vault_path(&self) -> &Path {
        &self.vault_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_country_page() {
        let reader = VaultReader::new(PathBuf::from("/tmp"));

        let content = r#"---
title: Finland
slug: fi
region: Northern Europe
aliases: [Suomi]
summary: Nordic country
updated_at: 2025-10-06
---

## Overview
Finland is a Nordic country.

### 2025-10-07 · politics · internal
[id:01J84N8P3E4]

Parliamentary debate continues.
"#;

        let page = reader.parse_country_page(content, "fi").unwrap();
        assert_eq!(page.country.title, "Finland");
        assert_eq!(page.notes.len(), 1);
        assert_eq!(page.notes[0].tags, vec!["politics"]);
    }
}
