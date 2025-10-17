mod commands;

use commands::{
    add_country_to_topic, add_note, add_topic_note, create_minimal_vault, create_topic,
    delete_note, delete_topic, delete_topic_note, get_all_countries_metadata,
    get_all_countries_with_combined_counts, get_all_country_stats, get_all_topics,
    get_country_metadata, get_country_notes, get_country_notes_with_topics, get_recent_activity,
    get_topic, get_topic_notes, get_topics_for_country, get_vault_manifest, list_countries,
    open_vault, remove_country_from_topic, search_notes, update_note, update_topic,
    update_topic_note,
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
            add_topic_note,
            get_topic_notes,
            update_topic_note,
            delete_topic_note,
            get_country_notes_with_topics,
            get_all_countries_with_combined_counts,
            get_recent_activity,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
