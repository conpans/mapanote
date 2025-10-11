use crate::models::{Note, Visibility};
use anyhow::Result;
use regex::Regex;

/// Parse notes from markdown body
pub fn parse_notes(markdown: &str) -> Result<Vec<Note>> {
    let mut notes = Vec::new();

    // Regex to match note headers
    // Format: ### 2025-10-07 · tags... · also:co,ve · internal · pinned
    let header_re = Regex::new(r"(?m)^###\s+(\d{4}-\d{2}-\d{2})\s*·\s*(.*?)\s*(?:·\s*(.*?))?\s*$")?;

    // Regex to match note IDs
    let id_re = Regex::new(r"\[id:([A-Z0-9]+)\]")?;

    let lines: Vec<&str> = markdown.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        // Check if this line is a note header
        if let Some(caps) = header_re.captures(line) {
            let date = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let tags_str = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            let metadata_str = caps.get(3).map(|m| m.as_str()).unwrap_or("");

            // Parse tags (comma or space separated)
            let tags: Vec<String> = tags_str
                .split(&[',', ' '][..])
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(String::from)
                .collect();

            // Parse metadata (also:co,ve · internal · pinned)
            let mut also = Vec::new();
            let mut visibility = Visibility::Internal;
            let mut pinned = false; // ← NEW

            for part in metadata_str.split('·') {
                let part = part.trim();

                if part.starts_with("also:") {
                    also = part[5..]
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                } else if part == "private" {
                    visibility = Visibility::Private;
                } else if part == "publishable" {
                    visibility = Visibility::Publishable;
                } else if part == "pinned" {
                    // ← NEW
                    pinned = true;
                }
            }

            // Collect note body (until next ### or end)
            i += 1;
            let mut body_lines = Vec::new();

            while i < lines.len() {
                let next_line = lines[i];

                // Stop at next note header
                if next_line.starts_with("###") {
                    break;
                }

                body_lines.push(next_line);
                i += 1;
            }

            let body = body_lines.join("\n").trim().to_string();

            // Extract ID from body
            let id = if let Some(id_caps) = id_re.captures(&body) {
                id_caps
                    .get(1)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default()
            } else {
                // Generate ID if missing
                ulid::Ulid::new().to_string()
            };

            // Remove ID marker from body text
            let text = id_re.replace(&body, "").trim().to_string();

            notes.push(Note {
                id,
                date: date.to_string(),
                tags,
                text,
                also,
                visibility,
                pinned, // ← NEW
            });
        } else {
            i += 1;
        }
    }

    Ok(notes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_notes() {
        let markdown = r#"
## Overview
Some intro text

### 2025-10-07 · politics, current · internal
[id:01J84N8P3E4]

Parliamentary debate over defense spending continues.

### 2025-10-06 · energy · also:se,no · publishable · pinned
[id:01J85P2K3M9]

Wind development plans announced.
"#;

        let notes = parse_notes(markdown).unwrap();
        assert_eq!(notes.len(), 2);

        let note1 = &notes[0];
        assert_eq!(note1.date, "2025-10-07");
        assert_eq!(note1.tags, vec!["politics", "current"]);
        assert!(!note1.pinned);

        let note2 = &notes[1];
        assert_eq!(note2.also, vec!["se", "no"]);
        assert!(note2.pinned);
    }
}
