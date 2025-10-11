// Prevents console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod models;
mod vault;

use commands::vault::{get_country, get_country_notes, list_countries, open_vault, AppState};
use std::sync::Mutex;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            vault_reader: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            open_vault,
            list_countries,
            get_country,
            get_country_notes,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
