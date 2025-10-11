use crate::models::{Country, Visibility};
use anyhow::{Context, Result};
use yaml_rust2::YamlLoader;

/// Extract frontmatter and body from markdown
pub fn split_frontmatter(content: &str) -> Result<(Option<String>, String)> {
    let content = content.trim();

    // Check if starts with ---
    if !content.starts_with("---") {
        return Ok((None, content.to_string()));
    }

    // Find the closing ---
    let after_first = &content[3..];
    if let Some(end_pos) = after_first.find("\n---") {
        let frontmatter = after_first[..end_pos].trim().to_string();
        let body = after_first[end_pos + 4..].trim().to_string();
        Ok((Some(frontmatter), body))
    } else {
        // No closing ---, treat whole thing as body
        Ok((None, content.to_string()))
    }
}

/// Parse YAML frontmatter into Country struct
pub fn parse_country_frontmatter(yaml_str: &str, slug: &str) -> Result<Country> {
    let docs = YamlLoader::load_from_str(yaml_str).context("Failed to parse YAML")?;

    if docs.is_empty() {
        anyhow::bail!("Empty YAML frontmatter");
    }

    let doc = &docs[0];

    // Helper to get string field
    let get_string = |key: &str| -> String { doc[key].as_str().unwrap_or("").to_string() };

    // Helper to get array field
    let get_array = |key: &str| -> Vec<String> {
        doc[key]
            .as_vec()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default()
    };

    Ok(Country {
        slug: slug.to_string(),
        title: get_string("title"),
        region: get_string("region"),
        summary: get_string("summary"),
        aliases: get_array("aliases"),
        updated_at: get_string("updated_at"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_frontmatter() {
        let content = r#"---
title: Finland
slug: fi
---

## Overview
Some content here
"#;

        let (fm, body) = split_frontmatter(content).unwrap();
        assert!(fm.is_some());
        assert!(body.contains("Overview"));
    }

    #[test]
    fn test_parse_frontmatter() {
        let yaml = r#"
title: Finland
slug: fi
region: Northern Europe
aliases: [Suomi, Republic of Finland]
summary: Nordic country
updated_at: 2025-10-06
"#;

        let country = parse_country_frontmatter(yaml, "fi").unwrap();
        assert_eq!(country.title, "Finland");
        assert_eq!(country.region, "Northern Europe");
        assert_eq!(country.aliases.len(), 2);
    }
}
