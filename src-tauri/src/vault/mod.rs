// Embed country metadata at compile time
const COUNTRY_METADATA: &str = include_str!("../../../src/lib/data/countries.json");

use once_cell::sync::Lazy;
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Deserialize)]
struct CountryMetadata {
    iso2: String,
    slug: String,
    name: String,
    region: String,
    subregion: String,
    summary: String,
    aliases: Vec<String>,
}

// Parse metadata once at startup
static COUNTRIES: Lazy<HashMap<String, CountryMetadata>> = Lazy::new(|| {
    let countries: Vec<CountryMetadata> =
        serde_json::from_str(COUNTRY_METADATA).expect("Failed to parse embedded country metadata");

    countries.into_iter().map(|c| (c.slug.clone(), c)).collect()
});

pub fn get_country_metadata(slug: &str) -> Option<&'static CountryMetadata> {
    COUNTRIES.get(slug)
}

mod frontmatter;
mod notes;
mod reader;
mod writer; // ← NEW

pub use reader::VaultReader;
pub use writer::VaultWriter; // ← NEW
