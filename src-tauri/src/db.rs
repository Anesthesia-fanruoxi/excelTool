use log::info;
use rusqlite::{params, Connection, Result as SqlResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Debug, Serialize, Deserialize)]
pub struct SalesRow {
    pub id: i64,
    pub data: String, // JSON 存储所有字段
    pub contract_no: String,
    pub customer: String,
    pub sale_date: String,
    pub amount: f64,
    pub profit: f64,
    pub status: String,
    pub created_at: String,
}

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new() -> SqlResult<Self> {
        let db_path = get_db_path();
        let conn = Connection::open(&db_path)?;
        
        // 创建表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS sales (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                data TEXT NOT NULL,
                contract_no TEXT NOT NULL,
                customer TEXT,
                sale_date TEXT,
                amount REAL DEFAULT 0,
                profit REAL DEFAULT 0,
                status TEXT,
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        // 创建索引
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_contract_no ON sales(contract_no)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_customer ON sales(customer)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_status ON sales(status)",
            [],
        )?;

        info!("Database initialized at {:?}", db_path);
        Ok(Self { conn: Mutex::new(conn) })
    }

    /// 清空销售表并释放磁盘空间
    pub fn clear_sales(&self) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM sales", [])?;
        conn.execute("VACUUM", [])?;  // 释放磁盘空间
        info!("Sales table cleared and vacuumed");
        Ok(())
    }

    /// 批量插入（导入时用）
    pub fn batch_insert(&self, rows: Vec<(String, String, String, String, f64, f64, String)>) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        let tx = conn.unchecked_transaction()?;
        let count = rows.len();
        
        for (data, contract_no, customer, sale_date, amount, profit, status) in rows {
            tx.execute(
                "INSERT INTO sales (data, contract_no, customer, sale_date, amount, profit, status, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, datetime('now'))",
                params![data, contract_no, customer, sale_date, amount, profit, status],
            )?;
        }
        
        tx.commit()?;
        info!("batch_insert committed {} rows to DB at {:?}", count, get_db_path());
        Ok(())
    }

    /// 分页查询
    pub fn query_page(&self, page: usize, page_size: usize, search: &str, status_filter: &str) -> SqlResult<Vec<SalesRow>> {
        let conn = self.conn.lock().unwrap();
        let offset = (page - 1) * page_size;
        
        let mut sql = "SELECT id, data, contract_no, customer, sale_date, amount, profit, status, created_at FROM sales WHERE 1=1".to_string();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![];
        
        if !search.is_empty() {
            sql.push_str(" AND (contract_no LIKE ?1 OR customer LIKE ?1 OR data LIKE ?1)");
            params_vec.push(Box::new(format!("%{}%", search)));
        }
        if !status_filter.is_empty() {
            let idx = params_vec.len() + 1;
            sql.push_str(&format!(" AND status = ?{}", idx));
            params_vec.push(Box::new(status_filter.to_string()));
        }
        
        sql.push_str(&format!(" ORDER BY id DESC LIMIT {} OFFSET {}", page_size, offset));
        
        let mut stmt = conn.prepare(&sql)?;
        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|b| b.as_ref()).collect();
        
        let rows = stmt.query_map(params_refs.as_slice(), |row| {
            Ok(SalesRow {
                id: row.get(0)?,
                data: row.get(1)?,
                contract_no: row.get(2)?,
                customer: row.get(3)?,
                sale_date: row.get(4)?,
                amount: row.get(5)?,
                profit: row.get(6)?,
                status: row.get(7)?,
                created_at: row.get(8)?,
            })
        })?;
        
        rows.collect()
    }

    /// 总数
    pub fn count(&self, search: &str, status_filter: &str) -> SqlResult<i64> {
        let conn = self.conn.lock().unwrap();
        let mut sql = "SELECT COUNT(*) FROM sales WHERE 1=1".to_string();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![];
        
        if !search.is_empty() {
            sql.push_str(" AND (contract_no LIKE ?1 OR customer LIKE ?1 OR data LIKE ?1)");
            params_vec.push(Box::new(format!("%{}%", search)));
        }
        if !status_filter.is_empty() {
            let idx = params_vec.len() + 1;
            sql.push_str(&format!(" AND status = ?{}", idx));
            params_vec.push(Box::new(status_filter.to_string()));
        }
        
        let mut stmt = conn.prepare(&sql)?;
        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|b| b.as_ref()).collect();
        stmt.query_row(params_refs.as_slice(), |row| row.get(0))
    }

    /// 按合同号聚合
    pub fn group_by_contract(&self, status_filter: &str) -> SqlResult<Vec<(String, String, String, String, i64, f64, f64)>> {
        let conn = self.conn.lock().unwrap();
        let mut sql = "SELECT contract_no, 
                              MAX(customer) as customer,
                              MAX(sale_date) as sale_date,
                              MAX(json_extract(data, '$.项目名称')) as project_name,
                              COUNT(*) as row_count,
                              SUM(amount) as total_amount,
                              SUM(profit) as total_profit
                       FROM sales WHERE 1=1".to_string();
        
        if !status_filter.is_empty() {
            sql.push_str(" AND status = ?1");
        }
        
        sql.push_str(" GROUP BY contract_no ORDER BY contract_no");
        
        let mut stmt = conn.prepare(&sql)?;
        let filter_params: Vec<&dyn rusqlite::ToSql> = if status_filter.is_empty() {
            vec![]
        } else {
            vec![&status_filter]
        };

        let rows = stmt.query_map(filter_params.as_slice(), |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, f64>(5)?,
                row.get::<_, f64>(6)?,
            ))
        })?;

        rows.collect()
    }

    /// 查询某合同的所有明细
    pub fn query_by_contract(&self, contract_no: &str) -> SqlResult<Vec<SalesRow>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, data, contract_no, customer, sale_date, amount, profit, status, created_at 
             FROM sales WHERE contract_no = ?1 ORDER BY id"
        )?;
        
        let rows = stmt.query_map([contract_no], |row| {
            Ok(SalesRow {
                id: row.get(0)?,
                data: row.get(1)?,
                contract_no: row.get(2)?,
                customer: row.get(3)?,
                sale_date: row.get(4)?,
                amount: row.get(5)?,
                profit: row.get(6)?,
                status: row.get(7)?,
                created_at: row.get(8)?,
            })
        })?;
        
        rows.collect()
    }

    /// 更新单行
    pub fn update_row(&self, id: i64, data: String, contract_no: String, customer: String, 
                      sale_date: String, amount: f64, profit: f64, status: String) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE sales SET data=?1, contract_no=?2, customer=?3, sale_date=?4, 
             amount=?5, profit=?6, status=?7 WHERE id=?8",
            params![data, contract_no, customer, sale_date, amount, profit, status, id],
        )?;
        Ok(())
    }

    /// 删除单行
    pub fn delete_row(&self, id: i64) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM sales WHERE id = ?1", [id])?;
        Ok(())
    }
}

fn get_db_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let app_dir = home.join(".excel-tool");
    std::fs::create_dir_all(&app_dir).ok();
    app_dir.join("sales.db")
}
