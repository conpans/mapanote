use mapanote_lib::models::CountryMetadata;
use std::sync::OnceLock;

// Embed countries.json at compile time
static COUNTRIES_DATA: &str = include_str!("../data/countries.json");

fn get_countries_metadata() -> &'static Vec<CountryMetadata> {
    static COUNTRIES: OnceLock<Vec<CountryMetadata>> = OnceLock::new();
    COUNTRIES.get_or_init(|| {
        serde_json::from_str(COUNTRIES_DATA).expect("Failed to parse embedded countries.json")
    })
}

#[tauri::command]
pub fn get_all_countries_metadata() -> Result<Vec<CountryMetadata>, String> {
    Ok(get_countries_metadata().clone())
}

#[tauri::command]
pub fn get_country_metadata(slug: String) -> Result<CountryMetadata, String> {
    get_countries_metadata()
        .iter()
        .find(|c| c.slug == slug)
        .cloned()
        .ok_or_else(|| format!("Country {} not found", slug))
}
