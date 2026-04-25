use crate::db::{Database, PageResult};
use crate::excel::{open_excel_file, read_sheet_data as read_excel_sheet, SheetInfo};
use log::info;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct ExcelData {
    pub sheets: Vec<SheetInfo>,
    pub file_path: String,
    pub file_name: String,
}

#[tauri::command]
pub fn open_excel(path: String) -> Result<ExcelData, String> {
    let sheets = open_excel_file(&path).map_err(|e| e.to_string())?;
    let file_name = std::path::Path::new(&path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("未知文件")
        .to_string();
    Ok(ExcelData { sheets, file_path: path, file_name })
}

#[tauri::command]
pub fn read_sheet_data(path: String, sheet_index: usize) -> Result<Vec<Vec<String>>, String> {
    read_excel_sheet(&path, sheet_index).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn import_sheet(
    db: tauri::State<Arc<Database>>,
    table_name: String,
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
) -> Result<usize, String> {
    db.create_tab_table(&table_name, &headers).map_err(|e| e.to_string())?;
    let count = db.insert_rows(&table_name, &headers, rows).map_err(|e| e.to_string())?;
    info!("import_sheet: table={} count={}", table_name, count);
    Ok(count)
}

#[tauri::command]
pub fn query_page(
    db: tauri::State<Arc<Database>>,
    table_name: String,
    page: usize,
    page_size: usize,
    filters: Vec<(String, String)>,
    col_filters: Vec<(String, Vec<String>)>,
) -> Result<PageResult, String> {
    db.query_page(&table_name, page, page_size, &filters, &col_filters)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_row_ids(
    db: tauri::State<Arc<Database>>,
    table_name: String,
    page: usize,
    page_size: usize,
    filters: Vec<(String, String)>,
    col_filters: Vec<(String, Vec<String>)>,
) -> Result<Vec<i64>, String> {
    db.get_row_ids(&table_name, page, page_size, &filters, &col_filters)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_distinct_values(
    db: tauri::State<Arc<Database>>,
    table_name: String,
    column: String,
) -> Result<Vec<(String, i64)>, String> {
    db.get_distinct_values(&table_name, &column)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_cell(
    db: tauri::State<Arc<Database>>,
    table_name: String,
    row_id: i64,
    column: String,
    value: String,
) -> Result<(), String> {
    db.update_cell(&table_name, row_id, &column, &value)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn drop_table(
    db: tauri::State<Arc<Database>>,
    table_name: String,
) -> Result<(), String> {
    db.drop_table(&table_name).map_err(|e| e.to_string())
}
