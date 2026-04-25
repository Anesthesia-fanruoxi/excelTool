use calamine::{open_workbook_auto, Data, Reader};
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct SheetInfo {
    pub name: String,
    pub index: usize,
    pub row_count: usize,
    pub col_count: usize,
}

#[derive(Error, Debug)]
pub enum ExcelError {
    #[error("Failed to open file: {0}")]
    OpenError(String),
    #[error("Failed to read sheet: {0}")]
    ReadError(String),
}

fn cell_to_string(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::String(s) => s.clone(),
        Data::Float(f) => {
            if f.fract() == 0.0 && f.abs() < 1e15 {
                format!("{}", *f as i64)
            } else {
                format!("{}", f)
            }
        }
        Data::Int(i) => format!("{}", i),
        Data::Bool(b) => if *b { "TRUE".to_string() } else { "FALSE".to_string() },
        Data::DateTime(dt) => excel_serial_to_date(dt.as_f64()),
        Data::DateTimeIso(s) => s.split('T').next().unwrap_or(s).replace('-', "/"),
        Data::DurationIso(s) => s.clone(),
        Data::Error(e) => format!("{:?}", e),
    }
}

fn excel_serial_to_date(serial: f64) -> String {
    if serial < 1.0 { return String::new(); }
    let days = if serial >= 60.0 { serial as i64 - 2 } else { serial as i64 - 1 };
    let epoch = chrono::NaiveDate::from_ymd_opt(1899, 12, 30).unwrap();
    match epoch.checked_add_signed(chrono::Duration::days(days)) {
        Some(date) => date.format("%Y/%m/%d").to_string(),
        None => format!("{}", serial as i64),
    }
}

pub fn open_excel_file(path: &str) -> Result<Vec<SheetInfo>, ExcelError> {
    let mut workbook =
        open_workbook_auto(path).map_err(|e| ExcelError::OpenError(e.to_string()))?;
    let sheets: Vec<SheetInfo> = workbook
        .sheet_names()
        .iter()
        .enumerate()
        .map(|(idx, name)| {
            let (row_count, col_count) = match workbook.worksheet_range(name) {
                Ok(range) => (range.height(), range.width()),
                Err(_) => (0, 0),
            };
            SheetInfo { name: name.clone(), index: idx, row_count, col_count }
        })
        .collect();
    info!("Found {} sheets", sheets.len());
    Ok(sheets)
}

/// 读取 sheet，返回 (所有行数据, 公式列map: col_index -> formula_str)
pub fn read_sheet_data(path: &str, sheet_index: usize) -> Result<(Vec<Vec<String>>, Vec<(usize, String)>), ExcelError> {
    let mut workbook =
        open_workbook_auto(path).map_err(|e| ExcelError::OpenError(e.to_string()))?;

    let sheet_names = workbook.sheet_names();
    let sheet_name = sheet_names
        .get(sheet_index)
        .ok_or_else(|| ExcelError::ReadError("Sheet not found".to_string()))?
        .clone();

    let range = workbook
        .worksheet_range(&sheet_name)
        .map_err(|e| ExcelError::ReadError(e.to_string()))?;

    let mut all_rows: Vec<Vec<String>> = range
        .rows()
        .map(|row| row.iter().map(cell_to_string).collect::<Vec<_>>())
        .filter(|row| row.iter().any(|c| !c.is_empty()))
        .collect();

    if all_rows.is_empty() {
        return Ok((all_rows, vec![]));
    }

    // 检测公式列（只看第一行数据）
    let mut formula_cols = HashSet::new();
    // 公式字符串 col_index -> formula_str
    let mut formula_map: std::collections::HashMap<usize, String> = std::collections::HashMap::new();

    if let Ok(frange) = workbook.worksheet_formula(&sheet_name) {
        let fstart_row = frange.start().map(|(r, _)| r).unwrap_or(0) as usize;
        let fstart_col = frange.start().map(|(_, c)| c).unwrap_or(0) as usize;
        let dstart_row = range.start().map(|(r, _)| r).unwrap_or(0) as usize;
        let dstart_col = range.start().map(|(_, c)| c).unwrap_or(0) as usize;

        info!("[formula] frange start=({},{}) drange start=({},{})",
            fstart_row, fstart_col, dstart_row, dstart_col);

        if all_rows.len() > 1 {
            let first_data_abs_row = dstart_row + 1;
            let frel_row = first_data_abs_row.saturating_sub(fstart_row);

            info!("[formula] checking first data row: abs={} frel={}", first_data_abs_row, frel_row);

            for c_idx in 0..all_rows[0].len() {
                let abs_col = dstart_col + c_idx;
                let frel_col = abs_col.saturating_sub(fstart_col);
                if let Some(f) = frange.get((frel_row, frel_col)) {
                    if !f.trim().is_empty() {
                        info!("[formula] col={} frel_col={} formula={:?}", c_idx, frel_col, f);
                        formula_cols.insert(c_idx);
                        formula_map.insert(c_idx, f.to_string());
                    }
                }
            }
        }
    } else {
        info!("[formula] worksheet_formula not available for sheet={}", sheet_name);
    }

    info!("[formula] detected {} formula cols: {:?}", formula_map.len(),
        formula_map.iter().map(|(k,v)| format!("col{}={}", k, v)).collect::<Vec<_>>());

    // 表头公式列加 [公式] 后缀
    if let Some(header) = all_rows.first_mut() {
        for &ci in &formula_cols {
            if let Some(h) = header.get_mut(ci) {
                if !h.ends_with("[公式]") {
                    h.push_str("[公式]");
                }
            }
        }
    }

    info!("[excel] sheet='{}' {} rows, {} formula cols", sheet_name, all_rows.len(), formula_map.len());

    let formulas: Vec<(usize, String)> = formula_map.into_iter().collect();
    Ok((all_rows, formulas))
}
