mod commands;

use commands::{
    add_country_to_topic, add_note, create_minimal_vault, create_topic, delete_note, delete_topic,
    get_all_countries_metadata, get_all_country_stats, get_all_topics, get_country_metadata,
    get_country_notes, get_topic, get_topics_for_country, get_vault_manifest, list_countries,
    open_vault, remove_country_from_topic, search_notes, update_note, update_topic,
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
            get_all_topics,
            get_topic,
            create_topic,
            update_topic,
            delete_topic,
            add_country_to_topic,
            remove_country_from_topic,
            get_topics_for_country,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
