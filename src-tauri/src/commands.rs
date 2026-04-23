use crate::db::{ContractDetailRow, ContractItemInput, ContractRow, Database, PriceHistory, QuoteItem, SalesItem};
use crate::excel::{open_excel_file, read_sheet_data as read_excel_sheet, SheetInfo};
use log::{info, warn};
use ring::digest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

// ── 公共结构 ──────────────────────────────────────────────

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
pub struct QuotePageResult {
    pub rows: Vec<QuoteItem>,
    pub total: i64,
    pub page: usize,
    pub page_size: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SalesPageResult {
    pub rows: Vec<SalesItem>,
    pub total: i64,
    pub page: usize,
    pub page_size: usize,
}

// ── Excel 列名 → 数据库字段 映射 ─────────────────────────

/// 报价表：Excel列名 → 数据库字段名（key 已经过 normalize_header 处理）
fn quote_col_map() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    // 新格式列名
    m.insert("货物名称",        "goods_name");
    m.insert("规格型号",        "spec");
    m.insert("单位",            "unit");
    m.insert("供应商",          "supplier");
    m.insert("税率",            "tax_rate");
    m.insert("成本单价（含税）", "cost_price");  // 全角括号，优先使用
    m.insert("日期",            "date");
    // 旧报价表兼容
    m.insert("列1",             "col1_raw");    // 原始含税前价格，用于计算 cost_price
    m
}

/// 销售表：Excel列名 → 数据库字段名
fn sales_col_map() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    // 新格式 / 旧销售表格式（列名一致，直接复用）
    m.insert("合同号",      "contract_no");
    m.insert("客户",        "customer");
    m.insert("销售日期",    "sale_date");
    m.insert("项目名称",    "project_name");
    m.insert("产品名称",    "product_name");
    m.insert("规格",        "spec");
    m.insert("数量",        "quantity");
    m.insert("单位",        "unit");
    m.insert("单价",        "unit_price");
    m.insert("供应商",      "supplier");
    m.insert("备注",        "remark");
    m
}

// ── UUID 生成 ─────────────────────────────────────────────

/// sha256(goods_name|spec) 取前16位十六进制
pub fn make_item_uuid(goods_name: &str, spec: &str) -> String {
    let input = format!("{}|{}", goods_name.trim(), spec.trim());
    let digest = digest::digest(&digest::SHA256, input.as_bytes());
    let hex: String = digest.as_ref().iter().map(|b| format!("{:02x}", b)).collect();
    hex[..16].to_string()
}

// ── 工具函数 ──────────────────────────────────────────────

fn normalize_header(h: &str) -> String {
    h.replace('\n', "")
     .replace('\r', "")
     .replace('(', "（")   // 半角转全角，统一匹配
     .replace(')', "）")
     .replace(' ', "")     // 去掉所有空格
     .trim()
     .to_string()
}

fn parse_f64(s: &str) -> f64 {
    s.replace(',', "").replace('%', "").trim().parse::<f64>().unwrap_or(0.0)
}

fn get_recent_files_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let app_dir = home.join(".excel-tool");
    fs::create_dir_all(&app_dir).ok();
    app_dir.join("recent.json")
}

// ── Excel 命令 ────────────────────────────────────────────

#[tauri::command]
pub fn open_excel(path: String) -> Result<ExcelData, String> {
    let sheets = open_excel_file(&path).map_err(|e| e.to_string())?;
    Ok(ExcelData { sheets, file_path: path })
}

#[tauri::command]
pub fn get_sheets(path: String) -> Result<Vec<SheetInfo>, String> {
    open_excel_file(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn read_sheet_data(path: String, sheet_index: usize) -> Result<Vec<Vec<String>>, String> {
    read_excel_sheet(&path, sheet_index).map_err(|e| e.to_string())
}

// ── 报价表命令 ────────────────────────────────────────────

#[tauri::command]
pub fn import_quote_items(
    db: tauri::State<Arc<Database>>,
    headers: Vec<String>,
    data: Vec<Vec<String>>,
) -> Result<usize, String> {
    let col_map = quote_col_map();
    // 标准化并映射表头：Excel列名 → 数据库字段名
    let mapped: Vec<Option<&str>> = headers.iter()
        .map(|h| col_map.get(normalize_header(h).as_str()).copied())
        .collect();

    // 打印映射结果，方便排查列名不匹配问题
    for (i, h) in headers.iter().enumerate() {
        info!("quote header[{}]: '{}' -> {:?}", i, h, mapped[i]);
    }

    db.clear_quote_items().map_err(|e| e.to_string())?;

    let rows: Vec<(String, String, String, String, String, String, f64, String)> = data
        .into_iter()
        .filter_map(|row| {
            let mut fields: HashMap<&str, String> = HashMap::new();
            for (i, field_opt) in mapped.iter().enumerate() {
                if let Some(field) = field_opt {
                    fields.insert(field, row.get(i).cloned().unwrap_or_default().trim().to_string());
                }
            }
            let goods_name = fields.get("goods_name").cloned().unwrap_or_default();
            if goods_name.is_empty() { return None; } // 跳过空行

            let spec       = fields.get("spec").cloned().unwrap_or_default();
            let unit       = fields.get("unit").cloned().unwrap_or_default();
            let supplier   = fields.get("supplier").cloned().unwrap_or_default();
            let tax_rate   = fields.get("tax_rate").cloned().unwrap_or_default();
            let date       = fields.get("date").cloned().unwrap_or_default();

            // cost_price 优先用直接列，否则从 列1 + 税率 计算
            // 旧报价表公式：税率>12% ? 列1 : 列1/0.87
            let cost_price = {
                let direct = parse_f64(fields.get("cost_price").map(|s| s.as_str()).unwrap_or(""));
                if direct > 0.0 {
                    direct
                } else {
                    let col1 = parse_f64(fields.get("col1_raw").map(|s| s.as_str()).unwrap_or(""));
                    if col1 > 0.0 {
                        let rate = parse_f64(tax_rate.replace('%', "").trim());
                        if rate > 12.0 { col1 } else { col1 / 0.87 }
                    } else {
                        0.0
                    }
                }
            };

            let uuid = make_item_uuid(&goods_name, &spec);

            Some((uuid, goods_name, spec, unit, supplier, tax_rate, cost_price, date))
        })
        .collect();

    let count = rows.len();
    db.batch_insert_quote_items(rows).map_err(|e| e.to_string())?;
    info!("import_quote_items: {} rows", count);
    Ok(count)
}

#[tauri::command]
pub fn get_quote_stats(db: tauri::State<Arc<Database>>) -> Result<TableStats, String> {
    let count = db.quote_items_count().map_err(|e| e.to_string())?;
    Ok(TableStats { name: "报价表".to_string(), count })
}

#[tauri::command]
pub fn clear_quote_table(db: tauri::State<Arc<Database>>) -> Result<(), String> {
    db.clear_quote_items().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn query_quote_page(
    db: tauri::State<Arc<Database>>,
    page: usize,
    page_size: usize,
    conditions: Vec<(String, String)>,
) -> Result<QuotePageResult, String> {
    let total = db.count_quote_items(&conditions).map_err(|e| e.to_string())?;
    let rows  = db.query_quote_items(page, page_size, &conditions).map_err(|e| e.to_string())?;
    Ok(QuotePageResult { rows, total, page, page_size })
}

#[tauri::command]
pub fn update_quote_item(
    db: tauri::State<Arc<Database>>,
    id: i64,
    unit: String,
    supplier: String,
    tax_rate: String,
    cost_price: f64,
) -> Result<(), String> {
    db.update_quote_item(id, unit, supplier, tax_rate, cost_price)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn query_price_history(
    db: tauri::State<Arc<Database>>,
    item_uuid: String,
) -> Result<Vec<PriceHistory>, String> {
    db.query_price_history(&item_uuid).map_err(|e| e.to_string())
}

// ── 销售表命令 ────────────────────────────────────────────

#[tauri::command]
pub fn import_sales_items(
    db: tauri::State<Arc<Database>>,
    headers: Vec<String>,
    data: Vec<Vec<String>>,
) -> Result<ImportSalesResult, String> {
    let col_map = sales_col_map();
    let mapped: Vec<Option<&str>> = headers.iter()
        .map(|h| col_map.get(normalize_header(h).as_str()).copied())
        .collect();

    // 打印映射结果
    for (i, h) in headers.iter().enumerate() {
        info!("sales header[{}]: '{}' -> {:?}", i, h, mapped[i]);
    }

    db.clear_sales_items().map_err(|e| e.to_string())?;

    // 预加载所有报价 UUID 用于匹配
    let all_quotes = db.query_quote_items(1, 999999, &[]).map_err(|e| e.to_string())?;
    let uuid_set: std::collections::HashSet<String> = all_quotes.iter().map(|q| q.uuid.clone()).collect();

    let mut linked = 0usize;
    let mut unlinked = 0usize;

    let rows = data.into_iter()
        .filter_map(|row| {
            let mut fields: HashMap<&str, String> = HashMap::new();
            for (i, field_opt) in mapped.iter().enumerate() {
                if let Some(field) = field_opt {
                    fields.insert(field, row.get(i).cloned().unwrap_or_default().trim().to_string());
                }
            }
            let contract_no = fields.get("contract_no").cloned().unwrap_or_default();
            if contract_no.is_empty() { return None; }

            let product_name = fields.get("product_name").cloned().unwrap_or_default();
            let spec         = fields.get("spec").cloned().unwrap_or_default();
            let quantity     = parse_f64(fields.get("quantity").map(|s| s.as_str()).unwrap_or(""));
            let unit_price   = parse_f64(fields.get("unit_price").map(|s| s.as_str()).unwrap_or(""));

            // 自动匹配报价 UUID
            let match_uuid = make_item_uuid(&product_name, &spec);
            let item_uuid: Option<String> = if uuid_set.contains(&match_uuid) {
                Some(match_uuid)
            } else {
                warn!("No quote match for: {} | {}", product_name, spec);
                None
            };

            Some((
                contract_no,
                fields.get("customer").cloned().unwrap_or_default(),
                fields.get("sale_date").cloned().unwrap_or_default(),
                fields.get("project_name").cloned().unwrap_or_default(),
                product_name,
                spec,
                quantity,
                fields.get("unit").cloned().unwrap_or_default(),
                unit_price,
                fields.get("supplier").cloned().unwrap_or_default(),
                fields.get("remark").cloned().unwrap_or_default(),
                item_uuid,
            ))
        })
        .collect::<Vec<_>>();

    // 统计关联情况
    for r in &rows {
        if r.11.is_some() { linked += 1; } else { unlinked += 1; }
    }

    let total = rows.len();
    db.batch_insert_sales_items(rows).map_err(|e| e.to_string())?;
    info!("import_sales_items: total={} linked={} unlinked={}", total, linked, unlinked);

    Ok(ImportSalesResult { total, linked, unlinked })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportSalesResult {
    pub total: usize,
    pub linked: usize,
    pub unlinked: usize,
}

#[tauri::command]
pub fn get_table_stats(db: tauri::State<Arc<Database>>) -> Result<TableStats, String> {
    let count = db.sales_items_count().map_err(|e| e.to_string())?;
    Ok(TableStats { name: "销售表".to_string(), count })
}

#[tauri::command]
pub fn clear_sales_table(db: tauri::State<Arc<Database>>) -> Result<(), String> {
    db.clear_sales_items().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn query_sales_page(
    db: tauri::State<Arc<Database>>,
    page: usize,
    page_size: usize,
    conditions: Vec<(String, String)>,
) -> Result<SalesPageResult, String> {
    let total = db.count_sales_items(&conditions).map_err(|e| e.to_string())?;
    let rows  = db.query_sales_items(page, page_size, &conditions).map_err(|e| e.to_string())?;
    Ok(SalesPageResult { rows, total, page, page_size })
}

#[tauri::command]
pub fn update_sales_item_price(
    db: tauri::State<Arc<Database>>,
    id: i64,
    quantity: f64,
    unit_price: f64,
) -> Result<(), String> {
    db.update_sales_item_price(id, quantity, unit_price)
        .map_err(|e| e.to_string())
}



#[tauri::command]
pub fn create_contract(
    db: tauri::State<Arc<Database>>,
    contract_no: String,
    customer: String,
    sale_date: String,
    project_name: String,
    items: Vec<ContractItemInput>,
) -> Result<usize, String> {
    db.create_contract(&contract_no, &customer, &sale_date, &project_name, items)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn query_contracts(
    db: tauri::State<Arc<Database>>,
    keyword: String,
) -> Result<Vec<ContractRow>, String> {
    db.query_contracts(&keyword).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn query_contract_detail(
    db: tauri::State<Arc<Database>>,
    contract_no: String,
) -> Result<Vec<ContractDetailRow>, String> {
    db.query_contract_detail(&contract_no).map_err(|e| e.to_string())
}

// ── 数据管理预览 ──────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct PreviewData {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub total: i64,
}

#[tauri::command]
pub fn preview_quote_data(
    db: tauri::State<Arc<Database>>,
    page: usize,
    page_size: usize,
) -> Result<PreviewData, String> {
    let headers = vec![
        "货物名称","规格型号","单位","供应商","税率","成本单价（含税）","日期","UUID"
    ].into_iter().map(String::from).collect::<Vec<_>>();

    let total = db.quote_items_count().map_err(|e| e.to_string())?;
    let items = db.query_quote_items(page, page_size, &[]).map_err(|e| e.to_string())?;
    let rows = items.into_iter().map(|q| vec![
        q.goods_name, q.spec, q.unit, q.supplier,
        q.tax_rate, q.cost_price.to_string(), q.date, q.uuid,
    ]).collect();

    Ok(PreviewData { headers, rows, total })
}

#[tauri::command]
pub fn preview_sales_data(
    db: tauri::State<Arc<Database>>,
    page: usize,
    page_size: usize,
) -> Result<PreviewData, String> {
    let headers = vec![
        "合同号","客户","销售日期","项目名称","产品名称",
        "规格","数量","单位","单价","供应商","备注","关联UUID"
    ].into_iter().map(String::from).collect::<Vec<_>>();

    let total = db.sales_items_count().map_err(|e| e.to_string())?;
    let items = db.query_sales_items(page, page_size, &[]).map_err(|e| e.to_string())?;
    let rows = items.into_iter().map(|s| vec![
        s.contract_no, s.customer, s.sale_date, s.project_name, s.product_name,
        s.spec, s.quantity.to_string(), s.unit, s.unit_price.to_string(),
        s.supplier, s.remark, s.item_uuid.unwrap_or_default(),
    ]).collect();

    Ok(PreviewData { headers, rows, total })
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
    } else { vec![] };
    files.retain(|f| f != &file_path);
    files.insert(0, file_path);
    if files.len() > 10 { files.truncate(10); }
    let json = serde_json::to_string_pretty(&files).map_err(|e| e.to_string())?;
    fs::write(&recent_path, json).map_err(|e| e.to_string())?;
    Ok(())
}
