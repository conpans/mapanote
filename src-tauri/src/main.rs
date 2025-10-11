// Prevents console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod models;
mod vault;

use commands::vault::{
    add_note,    // ← NEW
    delete_note, // ← NEW
    get_country,
    get_country_notes,
    list_countries,
    open_vault,
    update_note, // ← NEW
    AppState,
};
use std::sync::Mutex;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            vault_reader: Mutex::new(None),
            vault_writer: Mutex::new(None), // ← NEW
        })
        .invoke_handler(tauri::generate_handler![
            open_vault,
            list_countries,
            get_country,
            get_country_notes,
            add_note,    // ← NEW
            update_note, // ← NEW
            delete_note, // ← NEW
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
