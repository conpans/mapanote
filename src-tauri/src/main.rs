mod commands;

use commands::{
    add_note, create_minimal_vault, delete_note, get_all_countries_metadata, get_all_country_stats,
    get_country_metadata, get_country_notes, get_vault_manifest, list_countries, open_vault,
    search_notes, update_note,
};
use mapanote_lib::AppState;
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState {
            vault_reader: Mutex::new(None),
            vault_writer: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            open_vault,
            create_minimal_vault,
            list_countries,
            get_country_notes,
            get_vault_manifest,
            get_all_country_stats,
            get_all_countries_metadata,
            get_country_metadata,
            add_note,
            update_note,
            delete_note,
            search_notes,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
