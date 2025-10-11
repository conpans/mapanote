use crate::vault::VaultReader;
use crate::models::{Country, Note};
use tauri::State;
use std::sync::Mutex;
use std::path::PathBuf;

/// App state shared across commands
pub struct AppState {
    pub vault_reader: Mutex<Option<VaultReader>>,
}

#[tauri::command]
pub async fn open_vault(path: String, state: State<'_, AppState>) -> Result<String, String> {
    let vault_path = PathBuf::from(&path);
    let vault_reader = VaultReader::new(vault_path);
    
    // Verify vault exists by loading config
    match vault_reader.load_config() {
        Ok(config) => {
            *state.vault_reader.lock().unwrap() = Some(vault_reader);
            Ok(format!("Vault opened: schema v{}", config.schema_version))
        }
        Err(e) => Err(format!("Failed to open vault: {}", e)),
    }
}

#[tauri::command]
pub async fn list_countries(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let reader_guard = state.vault_reader.lock().unwrap();
    
    match reader_guard.as_ref() {
        Some(reader) => {
            reader.list_countries().map_err(|e| e.to_string())
        }
        None => Err("No vault opened".to_string()),
    }
}

#[tauri::command]
pub async fn get_country(slug: String, state: State<'_, AppState>) -> Result<Country, String> {
    let reader_guard = state.vault_reader.lock().unwrap();
    
    match reader_guard.as_ref() {
        Some(reader) => {
            let page = reader.read_country(&slug).map_err(|e| e.to_string())?;
            Ok(page.country)
        }
        None => Err("No vault opened".to_string()),
    }
}

#[tauri::command]
pub async fn get_country_notes(slug: String, state: State<'_, AppState>) -> Result<Vec<Note>, String> {
    let reader_guard = state.vault_reader.lock().unwrap();
    
    match reader_guard.as_ref() {
        Some(reader) => {
            let page = reader.read_country(&slug).map_err(|e| e.to_string())?;
            Ok(page.notes)
        }
        None => Err("No vault opened".to_string()),
    }
}