use log::info;
use rusqlite::{params, Connection, Result as SqlResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

pub struct Database {
    conn: Mutex<Connection>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PageResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<Option<String>>>,
    pub total: i64,
    pub page: usize,
    pub page_size: usize,
}

impl Database {
    pub fn new() -> SqlResult<Self> {
        let db_path = get_db_path();
        if db_path.exists() {
            let _ = std::fs::remove_file(&db_path);
        }
        let conn = Connection::open(&db_path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        info!("Database initialized at {:?}", db_path);
        Ok(Self { conn: Mutex::new(conn) })
    }

    pub fn create_tab_table(&self, table_name: &str, columns: &[String]) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(&format!("DROP TABLE IF EXISTS \"{}\"", table_name), [])?;
        let col_defs: String = columns
            .iter()
            .map(|c| format!("\"{}\" TEXT", c.replace('"', "")))
            .collect::<Vec<_>>()
            .join(", ");
        conn.execute_batch(&format!(
            "CREATE TABLE \"{}\" (\"__id\" INTEGER PRIMARY KEY AUTOINCREMENT, {})",
            table_name, col_defs
        ))?;
        info!("Created table: {} with {} columns", table_name, columns.len());
        Ok(())
    }

    pub fn insert_rows(
        &self,
        table_name: &str,
        columns: &[String],
        rows: Vec<Vec<String>>,
    ) -> SqlResult<usize> {
        let conn = self.conn.lock().unwrap();
        let col_names: String = columns
            .iter()
            .map(|c| format!("\"{}\"", c.replace('"', "")))
            .collect::<Vec<_>>()
            .join(", ");
        let placeholders: String = (1..=columns.len())
            .map(|i| format!("?{}", i))
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!("INSERT INTO \"{}\" ({}) VALUES ({})", table_name, col_names, placeholders);
        let tx = conn.unchecked_transaction()?;
        let count = rows.len();
        for row in rows {
            let vals: Vec<rusqlite::types::Value> = row
                .into_iter()
                .map(rusqlite::types::Value::Text)
                .collect();
            tx.execute(&sql, rusqlite::params_from_iter(vals.iter()))?;
        }
        tx.commit()?;
        info!("Inserted {} rows into {}", count, table_name);
        Ok(count)
    }

    pub fn get_distinct_values(&self, table_name: &str, column: &str) -> SqlResult<Vec<(String, i64)>> {
        let conn = self.conn.lock().unwrap();
        let col = column.replace('"', "");
        // 非空值
        let sql = format!(
            "SELECT \"{}\", COUNT(*) FROM \"{}\" WHERE \"{}\" IS NOT NULL AND \"{}\" != '' GROUP BY \"{}\" ORDER BY \"{}\" ASC",
            col, table_name, col, col, col, col
        );
        let mut stmt = conn.prepare(&sql)?;
        let mut vals: Vec<(String, i64)> = stmt
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))?
            .filter_map(|r| r.ok())
            .collect();
        // 空值数量
        let empty_count: i64 = conn.query_row(
            &format!("SELECT COUNT(*) FROM \"{}\" WHERE \"{}\" IS NULL OR \"{}\" = ''", table_name, col, col),
            [], |r| r.get(0),
        ).unwrap_or(0);
        if empty_count > 0 {
            vals.push(("__EMPTY__".to_string(), empty_count));
        }
        Ok(vals)
    }

    pub fn query_page(
        &self,
        table_name: &str,
        page: usize,
        page_size: usize,
        filters: &[(String, String)],
        col_filters: &[(String, Vec<String>)],
    ) -> SqlResult<PageResult> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(&format!("PRAGMA table_info(\"{}\")", table_name))?;
        let columns: Vec<String> = stmt
            .query_map([], |r| r.get::<_, String>(1))?
            .filter_map(|r| r.ok())
            .filter(|c| c != "__id")
            .collect();

        let where_clause = build_where(filters, col_filters);

        let total: i64 = conn.query_row(
            &format!("SELECT COUNT(*) FROM \"{}\" {}", table_name, where_clause),
            [], |r| r.get(0),
        )?;

        let offset = (page.saturating_sub(1)) * page_size;
        let col_select: String = columns
            .iter()
            .map(|c| format!("\"{}\"", c.replace('"', "")))
            .collect::<Vec<_>>()
            .join(", ");

        let sql = format!(
            "SELECT {} FROM \"{}\" {} ORDER BY __id DESC LIMIT {} OFFSET {}",
            col_select, table_name, where_clause, page_size, offset
        );

        let mut stmt = conn.prepare(&sql)?;
        let rows: Vec<Vec<Option<String>>> = stmt
            .query_map([], |r| {
                let mut row = Vec::new();
                for i in 0..columns.len() {
                    row.push(r.get::<_, Option<String>>(i)?);
                }
                Ok(row)
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(PageResult { columns, rows, total, page, page_size })
    }

    pub fn update_cell(&self, table_name: &str, row_id: i64, column: &str, value: &str) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            &format!("UPDATE \"{}\" SET \"{}\"=?1 WHERE __id=?2", table_name, column.replace('"', "")),
            params![value, row_id],
        )?;
        Ok(())
    }

    pub fn drop_table(&self, table_name: &str) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(&format!("DROP TABLE IF EXISTS \"{}\"", table_name), [])?;
        info!("Dropped table: {}", table_name);
        Ok(())
    }

    pub fn get_row_ids(&self, table_name: &str, page: usize, page_size: usize, filters: &[(String, String)], col_filters: &[(String, Vec<String>)]) -> SqlResult<Vec<i64>> {
        let conn = self.conn.lock().unwrap();
        let where_clause = build_where(filters, col_filters);
        let offset = (page.saturating_sub(1)) * page_size;
        let sql = format!(
            "SELECT __id FROM \"{}\" {} ORDER BY __id DESC LIMIT {} OFFSET {}",
            table_name, where_clause, page_size, offset
        );
        let mut stmt = conn.prepare(&sql)?;
        let ids: Vec<i64> = stmt
            .query_map([], |r| r.get(0))?
            .filter_map(|r| r.ok())
            .collect();
        Ok(ids)
    }
}

fn get_db_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let app_dir = home.join(".excel-tool");
    std::fs::create_dir_all(&app_dir).ok();
    app_dir.join("data.db")
}

/// 构建多条件 WHERE 子句，条件之间 AND
/// filters: (col, keyword) LIKE 模糊搜索
/// col_filters: (col, [val1, val2]) IN 精确筛选
fn build_where(filters: &[(String, String)], col_filters: &[(String, Vec<String>)]) -> String {
    let mut clauses: Vec<String> = vec![];

    for (col, kw) in filters {
        if col.is_empty() || kw.is_empty() { continue; }
        let clean = col.replace('"', "");
        clauses.push(format!("\"{}\" LIKE '%{}%'", clean, kw.replace('\'', "''")));
    }

    for (col, vals) in col_filters {
        if col.is_empty() || vals.is_empty() { continue; }
        let clean = col.replace('"', "");
        let has_empty = vals.iter().any(|v| v == "__EMPTY__");
        let non_empty: Vec<_> = vals.iter().filter(|v| v.as_str() != "__EMPTY__").collect();
        let mut sub = vec![];
        if !non_empty.is_empty() {
            let in_list = non_empty.iter()
                .map(|v| format!("'{}'", v.replace('\'', "''")))
                .collect::<Vec<_>>()
                .join(", ");
            sub.push(format!("\"{}\" IN ({})", clean, in_list));
        }
        if has_empty {
            sub.push(format!("(\"{}\" IS NULL OR \"{}\" = '')", clean, clean));
        }
        if !sub.is_empty() {
            clauses.push(format!("({})", sub.join(" OR ")));
        }
    }

    if clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", clauses.join(" AND "))
    }
}
