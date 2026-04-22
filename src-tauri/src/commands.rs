use crate::db::Database;
use crate::excel::{open_excel_file, read_sheet_data as read_excel_sheet, SheetInfo};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct ExcelData {
    pub sheets: Vec<SheetInfo>,
    pub file_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TableStats {
    pub name: String,
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PageResult {
    pub rows: Vec<HashMap<String, String>>,
    pub total: i64,
    pub page: usize,
    pub page_size: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContractSummary {
    pub contract_no: String,
    pub customer: String,
    pub sale_date: String,
    pub project_name: String,
    pub row_count: i64,
    pub total_amount: f64,
    pub total_profit: f64,
    pub reconcile_status: String,
}

fn get_recent_files_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let app_dir = home.join(".excel-tool");
    fs::create_dir_all(&app_dir).ok();
    app_dir.join("recent.json")
}

/// 标准化列名（保留备用）
#[allow(dead_code)]
fn normalize_header(h: &str) -> String {
    h.replace('\n', "")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == ' ' || c.is_ascii())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}

/// 计算状态列
fn calc_status(row: &HashMap<String, String>) -> String {
    let qty = row.get("数量").map(|s| s.trim()).unwrap_or("").to_string();
    if qty.is_empty() { return String::new(); }

    let sign_person = row.get("签收人").map(|s| s.trim()).unwrap_or("").to_string();
    let sign_date   = row.get("签收日期").map(|s| s.trim()).unwrap_or("").to_string();
    let reconcile   = row.get("与客户对账时间").map(|s| s.trim()).unwrap_or("").to_string();

    if !reconcile.is_empty() {
        "已对账".to_string()
    } else if !sign_person.is_empty() && !sign_date.is_empty() {
        "待对账".to_string()
    } else if !sign_person.is_empty() && sign_date.is_empty() {
        "回签不完整".to_string()
    } else {
        "等回签".to_string()
    }
}

/// 计算金额/利润
fn calc_amount(row: &HashMap<String, String>) -> (f64, f64) {
    let qty        = row.get("数量").and_then(|v| v.replace(',', "").trim().parse::<f64>().ok()).unwrap_or(0.0);
    let unit_price = row.get("单价").and_then(|v| v.replace(',', "").trim().parse::<f64>().ok()).unwrap_or(0.0);
    let cost_price = row.get("成本单价含税").and_then(|v| v.replace(',', "").trim().parse::<f64>().ok()).unwrap_or(0.0);

    let amount  = (unit_price * qty * 100.0).round() / 100.0;
    let payable = (cost_price * qty * 100.0).round() / 100.0;
    let profit  = ((amount - payable) * 100.0).round() / 100.0;
    (amount, profit)
}

// ── Excel 相关 ────────────────────────────────────────────

#[tauri::command]
pub fn open_excel(path: String) -> Result<ExcelData, String> {
    info!("Opening Excel file: {}", path);
    let sheets = open_excel_file(&path).map_err(|e| {
        error!("Failed to open Excel: {}", e);
        e.to_string()
    })?;
    Ok(ExcelData { sheets, file_path: path })
}

#[tauri::command]
pub fn get_sheets(path: String) -> Result<Vec<SheetInfo>, String> {
    open_excel_file(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn read_sheet_data(path: String, sheet_index: usize) -> Result<Vec<Vec<String>>, String> {
    info!("Reading sheet {} from {}", sheet_index, path);
    read_excel_sheet(&path, sheet_index).map_err(|e| e.to_string())
}

// ── 销售表 CRUD ───────────────────────────────────────────

/// 导入 Excel 数据到 SQLite（覆盖）
#[tauri::command]
pub fn import_sales(
    db: tauri::State<Arc<Database>>,
    headers: Vec<String>,
    data: Vec<Vec<String>>,
) -> Result<usize, String> {
    info!("Importing {} rows to SQLite", data.len());

    db.clear_sales().map_err(|e| e.to_string())?;

    let rows: Vec<(String, String, String, String, f64, f64, String)> = data
        .into_iter()
        .map(|row| {
            let mut map: HashMap<String, String> = HashMap::new();
            headers.iter().enumerate().for_each(|(i, h)| {
                map.insert(h.clone(), row.get(i).cloned().unwrap_or_default());
            });

            let contract_no = map.get("合同号").cloned().unwrap_or_default();
            let customer    = map.get("客户").cloned().unwrap_or_default();
            let sale_date   = map.get("销售日期").cloned().unwrap_or_default();
            let status      = calc_status(&map);
            let (amount, profit) = calc_amount(&map);
            let json = serde_json::to_string(&map).unwrap_or_default();

            (json, contract_no, customer, sale_date, amount, profit, status)
        })
        .collect();

    let count = rows.len();
    db.batch_insert(rows).map_err(|e| e.to_string())?;
    info!("Imported {} rows", count);
    Ok(count)
}

/// 分页查询销售表
#[tauri::command]
pub fn query_sales_page(
    db: tauri::State<Arc<Database>>,
    page: usize,
    page_size: usize,
    conditions: Vec<(String, String)>,
    status_filter: String,
) -> Result<PageResult, String> {
    let total = db.count(&conditions, &status_filter).map_err(|e| e.to_string())?;
    let db_rows = db.query_page(page, page_size, &conditions, &status_filter).map_err(|e| e.to_string())?;

    let rows: Vec<HashMap<String, String>> = db_rows
        .into_iter()
        .map(|r| {
            let mut map: HashMap<String, String> = serde_json::from_str(&r.data).unwrap_or_default();
            // 注入计算列
            let (amount, profit) = calc_amount(&map);
            map.insert("金额".to_string(), amount.to_string());
            map.insert("利润".to_string(), profit.to_string());
            map.insert("状态列".to_string(), calc_status(&map));
            map.insert("__id".to_string(), r.id.to_string());
            map
        })
        .collect();

    Ok(PageResult { rows, total, page, page_size })
}

/// 获取销售表统计
#[tauri::command]
pub fn get_table_stats(db: tauri::State<Arc<Database>>) -> Result<TableStats, String> {
    let count = db.count(&[], "").map_err(|e| e.to_string())?;
    Ok(TableStats { name: "销售表".to_string(), count })
}

/// 清空销售表
#[tauri::command]
pub fn clear_sales_table(db: tauri::State<Arc<Database>>) -> Result<(), String> {
    db.clear_sales().map_err(|e| e.to_string())
}

/// 新增/更新单行
#[tauri::command]
pub fn save_sales_row(
    db: tauri::State<Arc<Database>>,
    id: Option<i64>,
    row_data: HashMap<String, String>,
) -> Result<(), String> {
    let contract_no = row_data.get("合同号").cloned().unwrap_or_default();
    let customer    = row_data.get("客户").cloned().unwrap_or_default();
    let sale_date   = row_data.get("销售日期").cloned().unwrap_or_default();
    let status      = calc_status(&row_data);
    let (amount, profit) = calc_amount(&row_data);
    let json = serde_json::to_string(&row_data).map_err(|e| e.to_string())?;

    if let Some(row_id) = id {
        db.update_row(row_id, json, contract_no, customer, sale_date, amount, profit, status)
            .map_err(|e| e.to_string())?;
    } else {
        db.batch_insert(vec![(json, contract_no, customer, sale_date, amount, profit, status)])
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// 删除单行
#[tauri::command]
pub fn delete_sales_row(db: tauri::State<Arc<Database>>, id: i64) -> Result<(), String> {
    db.delete_row(id).map_err(|e| e.to_string())
}

/// 合同聚合查询
#[tauri::command]
pub fn query_contracts(
    db: tauri::State<Arc<Database>>,
    status_filter: String,
) -> Result<Vec<ContractSummary>, String> {
    let rows = db.group_by_contract(&status_filter).map_err(|e| e.to_string())?;

    let result = rows.into_iter().map(|(contract_no, customer, sale_date, project_name, row_count, total_amount, total_profit)| {
        // 合同对账状态：查该合同所有行的状态
        let reconcile_status = if let Ok(detail_rows) = db.query_by_contract(&contract_no) {
            let statuses: Vec<String> = detail_rows.iter().map(|r| r.status.clone()).collect();
            calc_contract_status(&statuses)
        } else {
            String::new()
        };

        ContractSummary {
            contract_no,
            customer,
            sale_date,
            project_name,
            row_count,
            total_amount: (total_amount * 100.0).round() / 100.0,
            total_profit: (total_profit * 100.0).round() / 100.0,
            reconcile_status,
        }
    }).collect();

    Ok(result)
}

/// 查询合同明细
#[tauri::command]
pub fn query_contract_detail(
    db: tauri::State<Arc<Database>>,
    contract_no: String,
) -> Result<Vec<HashMap<String, String>>, String> {
    let rows = db.query_by_contract(&contract_no).map_err(|e| e.to_string())?;
    let result = rows.into_iter().map(|r| {
        let mut map: HashMap<String, String> = serde_json::from_str(&r.data).unwrap_or_default();
        let (amount, profit) = calc_amount(&map);
        map.insert("金额".to_string(), amount.to_string());
        map.insert("利润".to_string(), profit.to_string());
        map.insert("状态列".to_string(), calc_status(&map));
        map.insert("__id".to_string(), r.id.to_string());
        map
    }).collect();
    Ok(result)
}

fn calc_contract_status(statuses: &[String]) -> String {
    let priority = |s: &str| match s {
        "已对账"    => 0,
        "待对账"    => 1,
        "等回签"    => 2,
        "回签不完整" => 3,
        _           => -1,
    };
    statuses.iter()
        .max_by_key(|s| priority(s.as_str()))
        .cloned()
        .unwrap_or_default()
}

// ── 预览数据（vault_status 兼容接口）────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct VaultEntry {
    pub name: String,
    pub headers: Vec<String>,
    pub data: Vec<Vec<String>>,
    pub imported_at: String,
    pub source_file: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VaultStatus {
    pub entries: Vec<VaultEntry>,
}

/// 返回销售表所有数据，供前端预览使用
#[tauri::command]
pub fn get_vault_status(db: tauri::State<Arc<Database>>) -> Result<VaultStatus, String> {
    let count = db.count(&[], "").map_err(|e| e.to_string())?;
    if count == 0 {
        return Ok(VaultStatus { entries: vec![] });
    }

    // 分批读取全部数据
    let batch = 1000usize;
    let total_pages = ((count as usize) + batch - 1) / batch;
    let mut all_rows: Vec<Vec<String>> = Vec::with_capacity(count as usize);

    // 收集所有列（按 SALES_COLUMNS 顺序）
    let headers: Vec<String> = vec![
        "客户","销售日期","合同号","送货单号","项目名称","收货地址",
        "序号","产品名称","规格","特征","数量","单位","单价","金额",
        "下单人","安装位置","备注","所属年份","签收人","签收日期",
        "与客户对账时间","状态列","供应商","初始报价","税率",
        "成本单价含税","应付金额","对账数量","对账单价","对账日期",
        "对账金额","对账备注","利润",
    ].into_iter().map(String::from).collect();

    for page in 1..=total_pages {
        let db_rows = db.query_page(page, batch, &[], "").map_err(|e| e.to_string())?;
        for r in db_rows {
            let mut map: std::collections::HashMap<String, String> =
                serde_json::from_str(&r.data).unwrap_or_default();
            // 注入计算列
            let (amount, profit) = calc_amount(&map);
            map.insert("金额".to_string(), amount.to_string());
            map.insert("利润".to_string(), profit.to_string());
            map.insert("状态列".to_string(), calc_status(&map));

            let row: Vec<String> = headers.iter()
                .map(|h| map.get(h).cloned().unwrap_or_default())
                .collect();
            all_rows.push(row);
        }
    }

    let entry = VaultEntry {
        name: "销售表".to_string(),
        headers,
        data: all_rows,
        imported_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        source_file: String::new(),
    };

    Ok(VaultStatus { entries: vec![entry] })
}

// ── 最近文件 ──────────────────────────────────────────────

#[tauri::command]
pub fn get_recent_files() -> Result<Vec<String>, String> {
    let path = get_recent_files_path();
    if !path.exists() { return Ok(vec![]); }
    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_recent_file(file_path: String) -> Result<(), String> {
    let recent_path = get_recent_files_path();
    let mut files: Vec<String> = if recent_path.exists() {
        let content = fs::read_to_string(&recent_path).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        vec![]
    };
    files.retain(|f| f != &file_path);
    files.insert(0, file_path);
    if files.len() > 10 { files.truncate(10); }
    let json = serde_json::to_string_pretty(&files).map_err(|e| e.to_string())?;
    fs::write(&recent_path, json).map_err(|e| e.to_string())?;
    Ok(())
}

