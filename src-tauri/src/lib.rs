pub mod models;

use std::sync::Mutex;

pub struct AppState {
    pub vault_reader: Mutex<Option<String>>,
    pub vault_writer: Mutex<Option<String>>,
}
