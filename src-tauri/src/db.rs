use log::info;
use rusqlite::{params, Connection, Result as SqlResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

pub struct Database {
    conn: Mutex<Connection>,
}

// ── 数据结构 ──────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct QuoteItem {
    pub id: i64,
    pub uuid: String,
    pub goods_name: String,
    pub spec: String,
    pub unit: String,
    pub supplier: String,
    pub tax_rate: String,
    pub cost_price: f64,
    pub date: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SalesItem {
    pub id: i64,
    pub contract_no: String,
    pub customer: String,
    pub sale_date: String,
    pub project_name: String,
    pub product_name: String,
    pub spec: String,
    pub quantity: f64,
    pub unit: String,
    pub unit_price: f64,
    pub supplier: String,
    pub remark: String,
    pub item_uuid: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContractRow {
    pub contract_no: String,
    pub customer: String,
    pub sale_date: String,
    pub project_name: String,
    pub product_count: i64,
    pub total_sale_amount: f64,
    pub total_cost_amount: f64,
    pub total_profit: f64,
    pub unlinked_count: i64,
}

impl Database {
    pub fn new() -> SqlResult<Self> {
        let db_path = get_db_path();
        let conn = Connection::open(&db_path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        let db = Self { conn: Mutex::new(conn) };
        db.migrate()?;
        info!("Database initialized at {:?}", db_path);
        Ok(db)
    }

    fn migrate(&self) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        info!("Running database migration...");

        conn.execute_batch("
            CREATE TABLE IF NOT EXISTS quote_items (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                uuid        TEXT NOT NULL UNIQUE,
                goods_name  TEXT NOT NULL,
                spec        TEXT NOT NULL DEFAULT '',
                unit        TEXT NOT NULL DEFAULT '',
                supplier    TEXT NOT NULL DEFAULT '',
                tax_rate    TEXT NOT NULL DEFAULT '',
                cost_price  REAL NOT NULL DEFAULT 0,
                date        TEXT NOT NULL DEFAULT '',
                created_at  TEXT NOT NULL,
                updated_at  TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_qi_uuid      ON quote_items(uuid);
            CREATE INDEX IF NOT EXISTS idx_qi_name      ON quote_items(goods_name);
            CREATE INDEX IF NOT EXISTS idx_qi_supplier  ON quote_items(supplier);

            CREATE TABLE IF NOT EXISTS quote_price_history (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                item_uuid   TEXT NOT NULL,
                cost_price  REAL,
                tax_rate    TEXT,
                supplier    TEXT,
                date        TEXT,
                created_at  TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_ph_uuid ON quote_price_history(item_uuid);

            CREATE TABLE IF NOT EXISTS sales_items (
                id           INTEGER PRIMARY KEY AUTOINCREMENT,
                contract_no  TEXT NOT NULL,
                customer     TEXT NOT NULL DEFAULT '',
                sale_date    TEXT NOT NULL DEFAULT '',
                project_name TEXT NOT NULL DEFAULT '',
                product_name TEXT NOT NULL DEFAULT '',
                spec         TEXT NOT NULL DEFAULT '',
                quantity     REAL NOT NULL DEFAULT 0,
                unit         TEXT NOT NULL DEFAULT '',
                unit_price   REAL NOT NULL DEFAULT 0,
                supplier     TEXT NOT NULL DEFAULT '',
                remark       TEXT NOT NULL DEFAULT '',
                item_uuid    TEXT,
                created_at   TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_si_contract  ON sales_items(contract_no);
            CREATE INDEX IF NOT EXISTS idx_si_customer  ON sales_items(customer);
            CREATE INDEX IF NOT EXISTS idx_si_uuid      ON sales_items(item_uuid);
        ")?;

        info!("Migration complete: quote_items, quote_price_history, sales_items tables ready");
        Ok(())
    }

    // ── 报价表 ────────────────────────────────────────────

    pub fn clear_quote_items(&self) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM quote_price_history", [])?;
        conn.execute("DELETE FROM quote_items", [])?;
        // VACUUM 不在此处调用，避免 WAL 模式下与事务冲突
        Ok(())
    }

    pub fn quote_items_count(&self) -> SqlResult<i64> {
        let conn = self.conn.lock().unwrap();
        conn.query_row("SELECT COUNT(*) FROM quote_items", [], |r| r.get(0))
    }

    pub fn batch_insert_quote_items(
        &self,
        rows: Vec<(String, String, String, String, String, String, f64, String)>,
        // (uuid, goods_name, spec, unit, supplier, tax_rate, cost_price, date)
    ) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        let tx = conn.unchecked_transaction()?;
        let count = rows.len();
        for (uuid, goods_name, spec, unit, supplier, tax_rate, cost_price, date) in rows {
            tx.execute(
                "INSERT INTO quote_items
                    (uuid, goods_name, spec, unit, supplier, tax_rate, cost_price, date, created_at, updated_at)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,datetime('now'),datetime('now'))",
                params![uuid, goods_name, spec, unit, supplier, tax_rate, cost_price, date],
            )?;
        }
        tx.commit()?;
        info!("batch_insert_quote_items: {} rows", count);
        Ok(())
    }

    pub fn query_quote_items(
        &self,
        page: usize,
        page_size: usize,
        conditions: &[(String, String)],
    ) -> SqlResult<Vec<QuoteItem>> {
        let conn = self.conn.lock().unwrap();
        let offset = (page - 1) * page_size;
        let (where_sql, params_vec) = build_quote_where(conditions);
        let sql = format!(
            "SELECT id,uuid,goods_name,spec,unit,supplier,tax_rate,cost_price,date
             FROM quote_items {where_sql}
             ORDER BY id DESC LIMIT {page_size} OFFSET {offset}"
        );
        let mut stmt = conn.prepare(&sql)?;
        let refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|b| b.as_ref()).collect();
        let rows = stmt.query_map(refs.as_slice(), map_quote_item)?;
        rows.collect()
    }

    pub fn count_quote_items(&self, conditions: &[(String, String)]) -> SqlResult<i64> {
        let conn = self.conn.lock().unwrap();
        let (where_sql, params_vec) = build_quote_where(conditions);
        let sql = format!("SELECT COUNT(*) FROM quote_items {where_sql}");
        let mut stmt = conn.prepare(&sql)?;
        let refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|b| b.as_ref()).collect();
        stmt.query_row(refs.as_slice(), |r| r.get(0))
    }

    // ── 销售表 ────────────────────────────────────────────

    pub fn clear_sales_items(&self) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM sales_items", [])?;
        // VACUUM 不在此处调用，避免 WAL 模式下与事务冲突
        Ok(())
    }

    pub fn sales_items_count(&self) -> SqlResult<i64> {
        let conn = self.conn.lock().unwrap();
        conn.query_row("SELECT COUNT(*) FROM sales_items", [], |r| r.get(0))
    }

    pub fn batch_insert_sales_items(
        &self,
        rows: Vec<(String, String, String, String, String, String, f64, String, f64, String, String, Option<String>)>,
        // (contract_no, customer, sale_date, project_name, product_name, spec, quantity, unit, unit_price, supplier, remark, item_uuid)
    ) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        let tx = conn.unchecked_transaction()?;
        let count = rows.len();
        for (contract_no, customer, sale_date, project_name, product_name,
             spec, quantity, unit, unit_price, supplier, remark, item_uuid) in rows
        {
            tx.execute(
                "INSERT INTO sales_items
                    (contract_no,customer,sale_date,project_name,product_name,
                     spec,quantity,unit,unit_price,supplier,remark,item_uuid,created_at)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,datetime('now'))",
                params![
                    contract_no, customer, sale_date, project_name, product_name,
                    spec, quantity, unit, unit_price, supplier, remark, item_uuid
                ],
            )?;
        }
        tx.commit()?;
        info!("batch_insert_sales_items: {} rows", count);
        Ok(())
    }

    pub fn query_sales_items(
        &self,
        page: usize,
        page_size: usize,
        conditions: &[(String, String)],
    ) -> SqlResult<Vec<SalesItem>> {
        let conn = self.conn.lock().unwrap();
        let offset = (page - 1) * page_size;
        let (where_sql, params_vec) = build_sales_where(conditions);
        let sql = format!(
            "SELECT id,contract_no,customer,sale_date,project_name,product_name,
                    spec,quantity,unit,unit_price,supplier,remark,item_uuid
             FROM sales_items {where_sql}
             ORDER BY id DESC LIMIT {page_size} OFFSET {offset}"
        );
        let mut stmt = conn.prepare(&sql)?;
        let refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|b| b.as_ref()).collect();
        let rows = stmt.query_map(refs.as_slice(), map_sales_item)?;
        rows.collect()
    }

    pub fn count_sales_items(&self, conditions: &[(String, String)]) -> SqlResult<i64> {
        let conn = self.conn.lock().unwrap();
        let (where_sql, params_vec) = build_sales_where(conditions);
        let sql = format!("SELECT COUNT(*) FROM sales_items {where_sql}");
        let mut stmt = conn.prepare(&sql)?;
        let refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|b| b.as_ref()).collect();
        stmt.query_row(refs.as_slice(), |r| r.get(0))
    }

    // ── 合同聚合 ──────────────────────────────────────────

    pub fn query_contracts(&self, keyword: &str) -> SqlResult<Vec<ContractRow>> {
        let conn = self.conn.lock().unwrap();

        let mut sql = "
            SELECT
                s.contract_no,
                MAX(s.customer)      AS customer,
                MAX(s.sale_date)     AS sale_date,
                MAX(s.project_name)  AS project_name,
                COUNT(*)             AS product_count,
                SUM(s.unit_price * s.quantity) AS total_sale,
                SUM(CASE WHEN s.item_uuid IS NOT NULL
                         THEN q.cost_price * s.quantity ELSE 0 END) AS total_cost,
                SUM(CASE WHEN s.item_uuid IS NOT NULL
                         THEN (s.unit_price - q.cost_price) * s.quantity ELSE 0 END) AS total_profit,
                SUM(CASE WHEN s.item_uuid IS NULL THEN 1 ELSE 0 END) AS unlinked_count
            FROM sales_items s
            LEFT JOIN quote_items q ON s.item_uuid = q.uuid
            WHERE 1=1".to_string();

        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![];
        if !keyword.is_empty() {
            sql.push_str(" AND (s.contract_no LIKE ?1 OR s.customer LIKE ?1 OR s.project_name LIKE ?1)");
            params_vec.push(Box::new(format!("%{}%", keyword)));
        }
        sql.push_str(" GROUP BY s.contract_no ORDER BY MAX(s.sale_date) DESC");

        let mut stmt = conn.prepare(&sql)?;
        let refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|b| b.as_ref()).collect();
        let rows = stmt.query_map(refs.as_slice(), |r| {
            Ok(ContractRow {
                contract_no:       r.get(0)?,
                customer:          r.get(1)?,
                sale_date:         r.get(2)?,
                project_name:      r.get(3)?,
                product_count:     r.get(4)?,
                total_sale_amount: round2(r.get::<_, f64>(5)?),
                total_cost_amount: round2(r.get::<_, f64>(6)?),
                total_profit:      round2(r.get::<_, f64>(7)?),
                unlinked_count:    r.get(8)?,
            })
        })?;
        rows.collect()
    }

    pub fn query_contract_detail(&self, contract_no: &str) -> SqlResult<Vec<ContractDetailRow>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("
            SELECT
                s.id, s.product_name, s.spec, s.quantity, s.unit,
                s.unit_price, s.supplier, s.remark, s.item_uuid,
                q.goods_name, q.cost_price, q.tax_rate
            FROM sales_items s
            LEFT JOIN quote_items q ON s.item_uuid = q.uuid
            WHERE s.contract_no = ?1
            ORDER BY s.id
        ")?;
        let rows = stmt.query_map([contract_no], |r| {
            let item_uuid: Option<String> = r.get(8)?;
            let cost_price: Option<f64>   = r.get(10)?;
            let sale_amount = round2(r.get::<_, f64>(5)? * r.get::<_, f64>(3)?);
            let cost_amount = match (item_uuid.as_ref(), cost_price) {
                (Some(_), Some(cp)) => Some(round2(cp * r.get::<_, f64>(3)?)),
                _ => None,
            };
            let profit = match cost_amount {
                Some(ca) => Some(round2(sale_amount - ca)),
                None => None,
            };
            Ok(ContractDetailRow {
                id:           r.get(0)?,
                product_name: r.get(1)?,
                spec:         r.get(2)?,
                quantity:     r.get(3)?,
                unit:         r.get(4)?,
                unit_price:   r.get(5)?,
                supplier:     r.get(6)?,
                remark:       r.get(7)?,
                item_uuid,
                goods_name:   r.get(9)?,
                cost_price,
                tax_rate:     r.get(11)?,
                sale_amount,
                cost_amount,
                profit,
            })
        })?;
        rows.collect()
    }

    /// 更新报价物品（先存历史，再更新）
    pub fn update_quote_item(
        &self,
        id: i64,
        unit: String,
        supplier: String,
        tax_rate: String,
        cost_price: f64,
    ) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        let (old_cost, old_tax, old_supplier, old_date): (f64, String, String, String) =
            conn.query_row(
                "SELECT cost_price, tax_rate, supplier, date FROM quote_items WHERE id=?1",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )?;
        let uuid: String = conn.query_row(
            "SELECT uuid FROM quote_items WHERE id=?1", [id], |r| r.get(0)
        )?;
        conn.execute(
            "INSERT INTO quote_price_history (item_uuid,cost_price,tax_rate,supplier,date,created_at)
             VALUES (?1,?2,?3,?4,?5,datetime('now'))",
            params![uuid, old_cost, old_tax, old_supplier, old_date],
        )?;
        // date 自动更新为当前时间
        conn.execute(
            "UPDATE quote_items SET unit=?1,supplier=?2,tax_rate=?3,cost_price=?4,date=datetime('now'),updated_at=datetime('now') WHERE id=?5",
            params![unit, supplier, tax_rate, cost_price, id],
        )?;
        Ok(())
    }

    /// 更新销售明细的数量和单价
    pub fn update_sales_item_price(
        &self,
        id: i64,
        quantity: f64,
        unit_price: f64,
    ) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE sales_items SET quantity=?1, unit_price=?2 WHERE id=?3",
            params![quantity, unit_price, id],
        )?;
        Ok(())
    }
    pub fn create_contract(
        &self,
        contract_no: &str,
        customer: &str,
        sale_date: &str,
        project_name: &str,
        items: Vec<ContractItemInput>,
    ) -> SqlResult<usize> {
        let conn = self.conn.lock().unwrap();
        // 检查合同号是否已存在
        let exists: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sales_items WHERE contract_no=?1",
            [contract_no], |r| r.get(0),
        )?;
        if exists > 0 {
            return Err(rusqlite::Error::InvalidParameterName(
                format!("合同号 {} 已存在", contract_no)
            ));
        }
        let tx = conn.unchecked_transaction()?;
        let count = items.len();
        for item in items {
            tx.execute(
                "INSERT INTO sales_items
                    (contract_no,customer,sale_date,project_name,product_name,
                     spec,quantity,unit,unit_price,supplier,remark,item_uuid,created_at)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,datetime('now'))",
                params![
                    contract_no, customer, sale_date, project_name,
                    item.product_name, item.spec, item.quantity, item.unit,
                    item.unit_price, item.supplier, item.remark, item.item_uuid
                ],
            )?;
        }
        tx.commit()?;
        Ok(count)
    }
    pub fn query_price_history(&self, item_uuid: &str) -> SqlResult<Vec<PriceHistory>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id,item_uuid,cost_price,tax_rate,supplier,date,created_at
             FROM quote_price_history WHERE item_uuid=?1 ORDER BY created_at DESC"
        )?;
        let rows = stmt.query_map([item_uuid], |r| {
            Ok(PriceHistory {
                id:         r.get(0)?,
                item_uuid:  r.get(1)?,
                cost_price: r.get(2)?,
                tax_rate:   r.get(3)?,
                supplier:   r.get(4)?,
                date:       r.get(5)?,
                created_at: r.get(6)?,
            })
        })?;
        rows.collect()
    }
}

// ── 辅助结构 ──────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct ContractItemInput {
    pub product_name: String,
    pub spec: String,
    pub quantity: f64,
    pub unit: String,
    pub unit_price: f64,
    pub supplier: String,
    pub remark: String,
    pub item_uuid: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContractDetailRow {
    pub id: i64,
    pub product_name: String,
    pub spec: String,
    pub quantity: f64,
    pub unit: String,
    pub unit_price: f64,
    pub supplier: String,
    pub remark: String,
    pub item_uuid: Option<String>,
    pub goods_name: Option<String>,
    pub cost_price: Option<f64>,
    pub tax_rate: Option<String>,
    pub sale_amount: f64,
    pub cost_amount: Option<f64>,
    pub profit: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PriceHistory {
    pub id: i64,
    pub item_uuid: String,
    pub cost_price: Option<f64>,
    pub tax_rate: Option<String>,
    pub supplier: Option<String>,
    pub date: Option<String>,
    pub created_at: String,
}

// ── 内部工具函数 ──────────────────────────────────────────

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

fn build_quote_where(conditions: &[(String, String)]) -> (String, Vec<Box<dyn rusqlite::ToSql>>) {
    let mut sql = String::from("WHERE 1=1");
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![];
    for (col, kw) in conditions {
        if kw.is_empty() { continue; }
        let idx = params.len() + 1;
        let col_expr = match col.as_str() {
            "goods_name" => "goods_name",
            "spec"       => "spec",
            "supplier"   => "supplier",
            "date"       => "date",
            _            => "goods_name",
        };
        sql.push_str(&format!(" AND {col_expr} LIKE ?{idx}"));
        params.push(Box::new(format!("%{}%", kw)));
    }
    (sql, params)
}

fn build_sales_where(conditions: &[(String, String)]) -> (String, Vec<Box<dyn rusqlite::ToSql>>) {
    let mut sql = String::from("WHERE 1=1");
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![];
    for (col, kw) in conditions {
        if kw.is_empty() { continue; }
        let idx = params.len() + 1;
        let col_expr = match col.as_str() {
            "contract_no"  => "contract_no",
            "customer"     => "customer",
            "product_name" => "product_name",
            "spec"         => "spec",
            "supplier"     => "supplier",
            _              => "contract_no",
        };
        sql.push_str(&format!(" AND {col_expr} LIKE ?{idx}"));
        params.push(Box::new(format!("%{}%", kw)));
    }
    (sql, params)
}

fn map_quote_item(r: &rusqlite::Row) -> SqlResult<QuoteItem> {
    Ok(QuoteItem {
        id:         r.get(0)?,
        uuid:       r.get(1)?,
        goods_name: r.get(2)?,
        spec:       r.get(3)?,
        unit:       r.get(4)?,
        supplier:   r.get(5)?,
        tax_rate:   r.get(6)?,
        cost_price: r.get(7)?,
        date:       r.get(8)?,
    })
}

fn map_sales_item(r: &rusqlite::Row) -> SqlResult<SalesItem> {
    Ok(SalesItem {
        id:           r.get(0)?,
        contract_no:  r.get(1)?,
        customer:     r.get(2)?,
        sale_date:    r.get(3)?,
        project_name: r.get(4)?,
        product_name: r.get(5)?,
        spec:         r.get(6)?,
        quantity:     r.get(7)?,
        unit:         r.get(8)?,
        unit_price:   r.get(9)?,
        supplier:     r.get(10)?,
        remark:       r.get(11)?,
        item_uuid:    r.get(12)?,
    })
}

fn get_db_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let app_dir = home.join(".excel-tool");
    std::fs::create_dir_all(&app_dir).ok();
    app_dir.join("sales.db")  // 保持与旧版本一致，避免数据丢失
}
