#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod db;
mod excel;
mod machine_id;

use db::Database;
use log::info;
use std::sync::Arc;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    info!("Excel Tool starting...");

    let db = Arc::new(Database::new().expect("Failed to initialize database"));

    tauri::Builder::default()
        .manage(db)
        .invoke_handler(tauri::generate_handler![
            commands::open_excel,
            commands::get_sheets,
            commands::read_sheet_data,
            commands::import_sales,
            commands::query_sales_page,
            commands::get_table_stats,
            commands::clear_sales_table,
            commands::save_sales_row,
            commands::delete_sales_row,
            commands::query_contracts,
            commands::query_contract_detail,
            commands::get_vault_status,
            commands::get_recent_files,
            commands::add_recent_file,
            commands::import_quote,
            commands::get_quote_stats,
            commands::clear_quote_table,
            commands::query_quote_page,
            commands::get_quote_vault_status,
            machine_id::get_machine_id,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
