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

#[derive(Debug, Serialize, Deserialize)]
pub struct SheetDataResult {
    pub rows: Vec<Vec<String>>,
    pub formulas: Vec<(usize, String)>,
}

#[tauri::command]
pub fn read_sheet_data(path: String, sheet_index: usize) -> Result<SheetDataResult, String> {
    let (rows, formulas) = read_excel_sheet(&path, sheet_index).map_err(|e| e.to_string())?;
    Ok(SheetDataResult { rows, formulas })
}

#[tauri::command]
pub fn import_sheet(
    db: tauri::State<Arc<Database>>,
    table_name: String,
    file_path: String,
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    formulas: Vec<(usize, String)>,
) -> Result<usize, String> {
    db.create_tab_table(&table_name, &headers).map_err(|e| e.to_string())?;
    let count = db.insert_rows(&table_name, &headers, rows).map_err(|e| e.to_string())?;
    if !formulas.is_empty() {
        db.save_formulas(&table_name, &formulas).map_err(|e| e.to_string())?;
    }
    if !file_path.is_empty() {
        db.save_file_path(&table_name, &file_path).map_err(|e| e.to_string())?;
        info!("import_sheet: saved file_path={}", file_path);
    }
    info!("import_sheet: table={} count={} formulas={}", table_name, count, formulas.len());
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
) -> Result<Vec<(usize, String)>, String> {
    let results = db.update_cell_and_recalc(&table_name, row_id, &column, &value)
        .map_err(|e| e.to_string())?;
    Ok(results)
}

#[tauri::command]
pub fn save_to_file(
    db: tauri::State<Arc<Database>>,
    table_name: String,
) -> Result<(), String> {
    match db.get_file_path(&table_name) {
        Ok(Some(file_path)) => {
            info!("save_to_file: writing to {}", file_path);
            write_excel_file(&db, &table_name, &file_path)
        }
        Ok(None) => Err("未找到关联文件路径".to_string()),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub fn export_excel(
    db: tauri::State<Arc<Database>>,
    table_name: String,
    save_path: String,
) -> Result<(), String> {
    write_excel_file(&db, &table_name, &save_path)
}

/// 将表数据写入 Excel 文件（供导出和自动写回共用）
fn write_excel_file(db: &Database, table_name: &str, save_path: &str) -> Result<(), String> {
    use rust_xlsxwriter::{Workbook, Format, Formula};

    let (columns, rows) = db.export_all(&table_name).map_err(|e| e.to_string())?;
    let formulas = db.load_formulas(&table_name).map_err(|e| e.to_string())?;
    // 公式列索引集合
    let formula_col_set: std::collections::HashSet<usize> = formulas.iter().map(|(i, _)| *i).collect();
    // col_index -> 原始公式模板（含行号，如 M3）
    let formula_map: std::collections::HashMap<usize, String> =
        formulas.into_iter().collect();

    let mut workbook = Workbook::new();
    let sheet = workbook.add_worksheet();
    let header_fmt = Format::new().set_bold();

    // 写表头（去掉 [公式] 后缀）
    for (ci, col) in columns.iter().enumerate() {
        let display = col.trim_end_matches("[公式]");
        sheet.write_with_format(0, ci as u16, display, &header_fmt)
            .map_err(|e| e.to_string())?;
    }

    // 写数据行，公式列写公式，普通列写值
    for (ri, row) in rows.iter().enumerate() {
        let excel_row = ri as u32 + 2; // Excel 行号，第1行是表头，数据从第2行开始
        for (ci, cell) in row.iter().enumerate() {
            if formula_col_set.contains(&ci) {
                if let Some(tmpl) = formula_map.get(&ci) {
                    // 把模板里的行号替换成当前行号
                    // 例: IF(M3="","",M3*K3) -> IF(M{excel_row}="","",M{excel_row}*K{excel_row})
                    let formula_str = replace_row_numbers(tmpl, excel_row);
                    sheet.write_formula(ri as u32 + 1, ci as u16, Formula::new(&formula_str))
                        .map_err(|e| e.to_string())?;
                }
            } else {
                let val = cell.as_deref().unwrap_or("");
                if let Ok(n) = val.parse::<f64>() {
                    sheet.write(ri as u32 + 1, ci as u16, n).map_err(|e| e.to_string())?;
                } else {
                    sheet.write(ri as u32 + 1, ci as u16, val).map_err(|e| e.to_string())?;
                }
            }
        }
    }

    workbook.save(&save_path).map_err(|e| e.to_string())?;
    info!("export_excel: saved to {}", save_path);
    Ok(())
}

/// 将公式模板中的行号替换为目标行号
/// 例: "IF(M3=\"\",\"\",M3*K3)" + row=5 -> "IF(M5=\"\",\"\",M5*K5)"
fn replace_row_numbers(formula: &str, target_row: u32) -> String {
    let mut result = String::new();
    let chars: Vec<char> = formula.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c.is_ascii_alphabetic() {
            // 收集连续字母（列名）
            let start = i;
            while i < chars.len() && chars[i].is_ascii_alphabetic() { i += 1; }
            let col_part: String = chars[start..i].iter().collect();
            // 后面跟数字则替换行号
            if i < chars.len() && chars[i].is_ascii_digit() {
                while i < chars.len() && chars[i].is_ascii_digit() { i += 1; }
                result.push_str(&col_part);
                result.push_str(&target_row.to_string());
            } else {
                result.push_str(&col_part);
            }
        } else {
            result.push(c);
            i += 1;
        }
    }
    result
}

#[tauri::command]
pub fn drop_table(
    db: tauri::State<Arc<Database>>,
    table_name: String,
) -> Result<(), String> {
    db.drop_table(&table_name).map_err(|e| e.to_string())
}
