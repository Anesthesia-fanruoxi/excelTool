#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod db;
mod excel;
mod formula;

use db::Database;
use log::info;
use std::sync::Arc;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    info!("App starting...");

    let db = Arc::new(Database::new().expect("Failed to initialize database"));

    tauri::Builder::default()
        .manage(db)
        .invoke_handler(tauri::generate_handler![
            commands::open_excel,
            commands::read_sheet_data,
            commands::import_sheet,
            commands::query_page,
            commands::get_row_ids,
            commands::get_distinct_values,
            commands::update_cell,
            commands::drop_table,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
