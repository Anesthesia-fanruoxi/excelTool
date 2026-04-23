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
    info!("App starting...");

    let db = Arc::new(Database::new().expect("Failed to initialize database"));

    tauri::Builder::default()
        .manage(db)
        .invoke_handler(tauri::generate_handler![
            // Excel
            commands::open_excel,
            commands::get_sheets,
            commands::read_sheet_data,
            // 报价表
            commands::import_quote_items,
            commands::get_quote_stats,
            commands::clear_quote_table,
            commands::query_quote_page,
            commands::update_quote_item,
            commands::query_price_history,
            // 销售表
            commands::import_sales_items,
            commands::get_table_stats,
            commands::clear_sales_table,
            commands::query_sales_page,
            commands::update_sales_item_price,
            // 合同管理
            commands::create_contract,
            commands::query_contracts,
            commands::query_contract_detail,
            // 数据预览
            commands::preview_quote_data,
            commands::preview_sales_data,
            // 最近文件
            commands::get_recent_files,
            commands::add_recent_file,
            // 机器码
            machine_id::get_machine_id,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
