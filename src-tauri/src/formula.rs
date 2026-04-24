/// 简单公式计算引擎
/// 支持：IF, IFS, YEAR, 四则运算, 字符串拼接(&), 比较运算
/// 列引用：A=col0, B=col1, ... 使用行数据 Vec<Option<String>> 按索引取值

use log::warn;

/// 将列字母转为索引，A=0, B=1, ..., Z=25, AA=26 ...
pub fn col_letter_to_index(s: &str) -> Option<usize> {
    let s = s.trim().to_uppercase();
    if s.is_empty() || !s.chars().all(|c| c.is_ascii_alphabetic()) {
        return None;
    }
    let mut idx: usize = 0;
    for c in s.chars() {
        idx = idx * 26 + (c as usize - 'A' as usize + 1);
    }
    Some(idx - 1)
}

/// 计算一行中所有公式列的值
/// formulas: Vec<(col_index, formula_str)>
/// row: 完整行数据（含公式列占位 None，按 all_col_count 长度）
pub fn compute_formula_cols(
    formulas: &[(usize, String)],
    row: &[Option<String>],
) -> Vec<(usize, String)> {
    formulas.iter().map(|(col_idx, formula)| {
        let val = eval_formula(formula, row);
        (*col_idx, val)
    }).collect()
}

/// 对外入口：计算单个公式
pub fn eval_formula(formula: &str, row: &[Option<String>]) -> String {
    let f = formula.trim();
    let f = if f.starts_with('=') { &f[1..] } else { f };
    eval_expr(f.trim(), row)
}

// ── 表达式求值 ────────────────────────────────────────────

fn eval_expr(expr: &str, row: &[Option<String>]) -> String {
    let expr = expr.trim();

    // 函数调用
    if let Some(result) = try_eval_func(expr, row) {
        return result;
    }

    // 字符串拼接 & （从右往左找，避免函数内部的 & 干扰）
    if let Some(result) = try_eval_concat(expr, row) {
        return result;
    }

    // 比较运算
    if let Some(result) = try_eval_compare(expr, row) {
        return result;
    }

    // 加减
    if let Some(result) = try_eval_additive(expr, row) {
        return result;
    }

    // 乘除
    if let Some(result) = try_eval_multiplicative(expr, row) {
        return result;
    }

    // 字符串字面量
    if expr.starts_with('"') && expr.ends_with('"') && expr.len() >= 2 {
        return expr[1..expr.len()-1].replace("\"\"", "\"");
    }

    // 数字字面量
    if let Ok(n) = expr.parse::<f64>() {
        return format_number(n);
    }

    // 列引用（如 A, B, AA，或带行号的 A1, B3 — 忽略行号）
    if expr.chars().next().map(|c| c.is_ascii_alphabetic()).unwrap_or(false) {
        // 取出前缀字母部分（忽略后面的数字行号）
        let letters: String = expr.chars().take_while(|c| c.is_ascii_alphabetic()).collect();
        if !letters.is_empty() {
            if let Some(idx) = col_letter_to_index(&letters) {
                return row.get(idx)
                    .and_then(|v| v.clone())
                    .unwrap_or_default();
            }
        }
    }

    warn!("[formula] unresolved expr: {:?}", expr);
    String::new()
}

fn format_number(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        format!("{}", n)
    }
}

// ── 函数解析 ─────────────────────────────────────────────

fn try_eval_func(expr: &str, row: &[Option<String>]) -> Option<String> {
    let upper = expr.to_uppercase();

    // _xlfn.IFS(...) 或 IFS(...)
    let func_name;
    let args_str;
    if let Some(rest) = upper.strip_prefix("_XLFN.") {
        let paren = rest.find('(')?;
        func_name = rest[..paren].to_string();
        let inner = expr.find('(')? + 1;
        args_str = &expr[inner..expr.len()-1];
    } else {
        let paren = upper.find('(')?;
        // 确保最后一个字符是 )
        if !expr.ends_with(')') { return None; }
        func_name = upper[..paren].to_string();
        args_str = &expr[paren+1..expr.len()-1];
    }

    let args = split_args(args_str);

    match func_name.as_str() {
        "IF" => {
            if args.len() < 2 { return Some(String::new()); }
            let cond = eval_bool(&args[0], row);
            if cond {
                Some(eval_expr(&args[1], row))
            } else {
                Some(args.get(2).map(|a| eval_expr(a, row)).unwrap_or_default())
            }
        }
        "IFS" => {
            // IFS(cond1, val1, cond2, val2, ...)
            let mut i = 0;
            while i + 1 < args.len() {
                let cond_str = args[i].trim();
                // TRUE 作为最后的默认条件
                if cond_str.to_uppercase() == "TRUE" {
                    return Some(eval_expr(&args[i+1], row));
                }
                if eval_bool(cond_str, row) {
                    return Some(eval_expr(&args[i+1], row));
                }
                i += 2;
            }
            Some(String::new())
        }
        "YEAR" => {
            if args.is_empty() { return Some(String::new()); }
            let val = eval_expr(&args[0], row);
            if val.is_empty() { return Some(String::new()); }
            // 支持 "2026/04/15" 或 "2026-04-15"
            let year = val.split(['/', '-']).next().unwrap_or("").trim().to_string();
            Some(year)
        }
        "AND" => {
            let result = args.iter().all(|a| eval_bool(a.trim(), row));
            Some(if result { "TRUE".to_string() } else { "FALSE".to_string() })
        }
        "OR" => {
            let result = args.iter().any(|a| eval_bool(a.trim(), row));
            Some(if result { "TRUE".to_string() } else { "FALSE".to_string() })
        }
        _ => {
            warn!("[formula] unsupported function: {}", func_name);
            Some(String::new())
        }
    }
}

// ── 布尔求值 ─────────────────────────────────────────────

fn eval_bool(expr: &str, row: &[Option<String>]) -> bool {
    let expr = expr.trim();
    let upper = expr.to_uppercase();

    if upper == "TRUE" { return true; }
    if upper == "FALSE" { return false; }

    // AND(...) / OR(...)
    if upper.starts_with("AND(") || upper.starts_with("OR(") {
        let val = eval_expr(expr, row);
        return val.to_uppercase() == "TRUE";
    }

    // 比较运算
    for op in &["<>", ">=", "<=", ">", "<", "="] {
        if let Some(pos) = find_op_outside_parens(expr, op) {
            let left = eval_expr(expr[..pos].trim(), row);
            let right = eval_expr(expr[pos+op.len()..].trim(), row);
            return match *op {
                "="  => left == right,
                "<>" => left != right,
                ">"  => cmp_vals(&left, &right) == std::cmp::Ordering::Greater,
                "<"  => cmp_vals(&left, &right) == std::cmp::Ordering::Less,
                ">=" => cmp_vals(&left, &right) != std::cmp::Ordering::Less,
                "<=" => cmp_vals(&left, &right) != std::cmp::Ordering::Greater,
                _    => false,
            };
        }
    }

    // 非空即 true
    let val = eval_expr(expr, row);
    !val.is_empty() && val.to_uppercase() != "FALSE"
}

fn cmp_vals(a: &str, b: &str) -> std::cmp::Ordering {
    match (a.parse::<f64>(), b.parse::<f64>()) {
        (Ok(fa), Ok(fb)) => fa.partial_cmp(&fb).unwrap_or(std::cmp::Ordering::Equal),
        _ => a.cmp(b),
    }
}

// ── 运算符解析 ───────────────────────────────────────────

fn try_eval_concat(expr: &str, row: &[Option<String>]) -> Option<String> {
    let pos = find_op_outside_parens(expr, "&")?;
    let left = eval_expr(expr[..pos].trim(), row);
    let right = eval_expr(expr[pos+1..].trim(), row);
    Some(format!("{}{}", left, right))
}

fn try_eval_compare(expr: &str, row: &[Option<String>]) -> Option<String> {
    for op in &["<>", ">=", "<=", ">", "<", "="] {
        if let Some(pos) = find_op_outside_parens(expr, op) {
            let left = eval_expr(expr[..pos].trim(), row);
            let right = eval_expr(expr[pos+op.len()..].trim(), row);
            let result = match *op {
                "="  => left == right,
                "<>" => left != right,
                ">"  => cmp_vals(&left, &right) == std::cmp::Ordering::Greater,
                "<"  => cmp_vals(&left, &right) == std::cmp::Ordering::Less,
                ">=" => cmp_vals(&left, &right) != std::cmp::Ordering::Less,
                "<=" => cmp_vals(&left, &right) != std::cmp::Ordering::Greater,
                _    => false,
            };
            return Some(if result { "TRUE".to_string() } else { "FALSE".to_string() });
        }
    }
    None
}

fn try_eval_additive(expr: &str, row: &[Option<String>]) -> Option<String> {
    // 从右往左找 + 或 - （跳过括号内）
    for op in &["+", "-"] {
        if let Some(pos) = find_op_outside_parens_rightmost(expr, op) {
            if pos == 0 { continue; } // 负号
            let left = eval_expr(expr[..pos].trim(), row);
            let right = eval_expr(expr[pos+1..].trim(), row);
            let lf = left.parse::<f64>().ok()?;
            let rf = right.parse::<f64>().ok()?;
            return Some(format_number(if *op == "+" { lf + rf } else { lf - rf }));
        }
    }
    None
}

fn try_eval_multiplicative(expr: &str, row: &[Option<String>]) -> Option<String> {
    for op in &["*", "/"] {
        if let Some(pos) = find_op_outside_parens_rightmost(expr, op) {
            let left = eval_expr(expr[..pos].trim(), row);
            let right = eval_expr(expr[pos+1..].trim(), row);
            let lf = left.parse::<f64>().ok()?;
            let rf = right.parse::<f64>().ok()?;
            if *op == "/" && rf == 0.0 { return Some(String::new()); }
            return Some(format_number(if *op == "*" { lf * rf } else { lf / rf }));
        }
    }
    None
}

// ── 工具函数 ─────────────────────────────────────────────

/// 在括号外查找操作符（从左往右第一个）
fn find_op_outside_parens(expr: &str, op: &str) -> Option<usize> {
    let bytes = expr.as_bytes();
    let mut depth = 0i32;
    let mut in_str = false;
    let op_bytes = op.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'"' { in_str = !in_str; }
        if in_str { i += 1; continue; }
        if b == b'(' { depth += 1; }
        else if b == b')' { depth -= 1; }
        else if depth == 0 && bytes[i..].starts_with(op_bytes) {
            return Some(i);
        }
        i += 1;
    }
    None
}

/// 在括号外查找操作符（从右往左第一个）
fn find_op_outside_parens_rightmost(expr: &str, op: &str) -> Option<usize> {
    let bytes = expr.as_bytes();
    let op_bytes = op.as_bytes();
    let mut depth = 0i32;
    let mut in_str = false;
    let mut result = None;
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'"' { in_str = !in_str; }
        if in_str { i += 1; continue; }
        if b == b'(' { depth += 1; }
        else if b == b')' { depth -= 1; }
        else if depth == 0 && bytes[i..].starts_with(op_bytes) {
            result = Some(i);
        }
        i += 1;
    }
    result
}

/// 按逗号分割参数（忽略括号和字符串内的逗号）
fn split_args(s: &str) -> Vec<&str> {
    let mut args = Vec::new();
    let mut depth = 0i32;
    let mut in_str = false;
    let mut start = 0;
    let bytes = s.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        if b == b'"' { in_str = !in_str; }
        if in_str { continue; }
        if b == b'(' { depth += 1; }
        else if b == b')' { depth -= 1; }
        else if b == b',' && depth == 0 {
            args.push(s[start..i].trim());
            start = i + 1;
        }
    }
    args.push(s[start..].trim());
    args
}
