use calamine::{open_workbook_auto, Data, Reader};
use log::info;
use serde::{Deserialize, Serialize};
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

/// 将 Excel 单元格转为字符串，日期类型格式化为 YYYY/MM/DD
fn cell_to_string(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::String(s) => s.clone(),
        Data::Float(f) => {
            // 整数浮点去掉小数点
            if f.fract() == 0.0 && f.abs() < 1e15 {
                format!("{}", *f as i64)
            } else {
                format!("{}", f)
            }
        }
        Data::Int(i) => format!("{}", i),
        Data::Bool(b) => if *b { "TRUE".to_string() } else { "FALSE".to_string() },
        Data::DateTime(dt) => {
            // calamine 0.24 ExcelDateTime 只有 as_f64()，手动转换序列号
            excel_serial_to_date(dt.as_f64())
        }
        Data::DateTimeIso(s) => {
            s.split('T').next().unwrap_or(s).replace('-', "/")
        }
        Data::DurationIso(s) => s.clone(),
        Data::Error(e) => format!("{:?}", e),
    }
}

/// Excel 日期序列号转 YYYY/MM/DD
fn excel_serial_to_date(serial: f64) -> String {
    if serial < 1.0 {
        return String::new();
    }
    // Excel 1900 日期系统：epoch 是 1899/12/30
    // 序列号 60 对应不存在的 1900/2/29（Excel bug），60以后需要减1
    let days = if serial >= 60.0 {
        serial as i64 - 2
    } else {
        serial as i64 - 1
    };
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
            SheetInfo {
                name: name.clone(),
                index: idx,
                row_count,
                col_count,
            }
        })
        .collect();

    info!("Found {} sheets", sheets.len());
    Ok(sheets)
}

pub fn read_sheet_data(path: &str, sheet_index: usize) -> Result<Vec<Vec<String>>, ExcelError> {
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

    let rows: Vec<Vec<String>> = range
        .rows()
        .map(|row: &[Data]| row.iter().map(cell_to_string).collect::<Vec<String>>())
        .filter(|row| row.iter().any(|cell| !cell.trim().is_empty()))
        .collect();

    info!("Read {} rows from sheet {}", rows.len(), sheet_name);
    Ok(rows)
}
