# 数据库设计文档

> 版本：v2.0  
> 更新日期：2026-04-23

---

## 一、整体架构

系统分为三个业务模块，对应三张核心数据表：

```
报价明细（对接供应商）  →  quote_items + quote_price_history
销售明细（对接客户）    →  sales_items
合同管理（成交汇总）    →  sales_items JOIN quote_items（聚合视图，无独立表）
```

---

## 二、数据库表设计

### 2.1 报价物品表 `quote_items`

存储供应商报价的物品信息，每个物品由 `uuid` 唯一标识。

```sql
CREATE TABLE quote_items (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    uuid        TEXT NOT NULL UNIQUE,        -- 由 goods_name + spec 生成的 sha256 前16位
    goods_name  TEXT NOT NULL,               -- 货物名称
    spec        TEXT NOT NULL DEFAULT '',    -- 规格型号
    unit        TEXT,                        -- 单位
    supplier    TEXT,                        -- 供应商
    tax_rate    TEXT,                        -- 税率（原始字符串，如 "13%"）
    cost_price  REAL,                        -- 成本单价（含税）
    date        TEXT,                        -- 报价日期
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);

CREATE INDEX idx_quote_items_uuid ON quote_items(uuid);
CREATE INDEX idx_quote_items_goods_name ON quote_items(goods_name);
CREATE INDEX idx_quote_items_supplier ON quote_items(supplier);
```

### 2.2 历史价格表 `quote_price_history`

每次导入报价表时，若物品已存在且价格发生变化，则写入一条历史记录。

```sql
CREATE TABLE quote_price_history (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    item_uuid   TEXT NOT NULL,               -- 关联 quote_items.uuid
    cost_price  REAL,                        -- 当时的成本单价（含税）
    tax_rate    TEXT,                        -- 当时的税率
    supplier    TEXT,                        -- 当时的供应商
    date        TEXT,                        -- 报价日期
    created_at  TEXT NOT NULL
);

CREATE INDEX idx_price_history_uuid ON quote_price_history(item_uuid);
```

### 2.3 销售明细表 `sales_items`

存储销售订单明细，每行代表一个产品的销售记录。

```sql
CREATE TABLE sales_items (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    contract_no  TEXT NOT NULL,              -- 合同号
    customer     TEXT,                       -- 客户
    sale_date    TEXT,                       -- 销售日期
    project_name TEXT,                       -- 项目名称
    product_name TEXT,                       -- 产品名称（客户侧叫法）
    spec         TEXT,                       -- 规格
    quantity     REAL,                       -- 数量
    unit         TEXT,                       -- 单位
    unit_price   REAL,                       -- 销售单价（含税，卖给客户的价格）
    supplier     TEXT,                       -- 供应商
    remark       TEXT,                       -- 备注
    item_uuid    TEXT,                       -- 关联 quote_items.uuid（自动匹配，可为空）
    created_at   TEXT NOT NULL
);

CREATE INDEX idx_sales_contract ON sales_items(contract_no);
CREATE INDEX idx_sales_customer ON sales_items(customer);
CREATE INDEX idx_sales_item_uuid ON sales_items(item_uuid);
```

---

## 三、字段映射关系

### 3.1 报价表 Excel → 数据库

导入时只读取以下列，其他列自动忽略。

| Excel 列名 | 数据库字段 | 类型 | 说明 |
|-----------|-----------|------|------|
| 货物名称 | `goods_name` | TEXT | 必填，参与 UUID 生成 |
| 规格型号 | `spec` | TEXT | 参与 UUID 生成，可为空 |
| 单位 | `unit` | TEXT | |
| 供应商 | `supplier` | TEXT | |
| 税率 | `tax_rate` | TEXT | 原始字符串存储，如 "13%" 或 "13" |
| 成本单价（含税） | `cost_price` | REAL | |
| 日期 | `date` | TEXT | |

### 3.2 销售表 Excel → 数据库

导入时只读取以下列，其他列自动忽略。

| Excel 列名 | 数据库字段 | 类型 | 说明 |
|-----------|-----------|------|------|
| 合同号 | `contract_no` | TEXT | 必填 |
| 客户 | `customer` | TEXT | |
| 销售日期 | `sale_date` | TEXT | |
| 项目名称 | `project_name` | TEXT | |
| 产品名称 | `product_name` | TEXT | 参与 UUID 自动匹配 |
| 规格 | `spec` | TEXT | 参与 UUID 自动匹配 |
| 数量 | `quantity` | REAL | |
| 单位 | `unit` | TEXT | |
| 单价 | `unit_price` | REAL | 销售单价（卖给客户的价格） |
| 供应商 | `supplier` | TEXT | |
| 备注 | `remark` | TEXT | |

### 3.3 数据库字段 → 前端显示

前端展示时统一使用中文标签，通过映射表转换。

**报价明细前端列：**

| 数据库字段 | 前端显示名 |
|-----------|-----------|
| `goods_name` | 货物名称 |
| `spec` | 规格型号 |
| `unit` | 单位 |
| `supplier` | 供应商 |
| `tax_rate` | 税率 |
| `cost_price` | 成本单价（含税） |
| `date` | 日期 |

**销售明细前端列：**

| 数据库字段 | 前端显示名 |
|-----------|-----------|
| `contract_no` | 合同号 |
| `customer` | 客户 |
| `sale_date` | 销售日期 |
| `project_name` | 项目名称 |
| `product_name` | 产品名称 |
| `spec` | 规格 |
| `quantity` | 数量 |
| `unit` | 单位 |
| `unit_price` | 单价 |
| `supplier` | 供应商 |
| `remark` | 备注 |
| `item_uuid` | 关联报价UUID |

**合同管理前端列（聚合计算）：**

| 字段来源 | 前端显示名 | 计算方式 |
|---------|-----------|---------|
| `sales_items.contract_no` | 合同号 | 聚合键 |
| `sales_items.customer` | 客户 | MAX |
| `sales_items.sale_date` | 销售日期 | MAX |
| `sales_items.project_name` | 项目名称 | MAX |
| COUNT(*) | 产品数 | 计数 |
| SUM(unit_price × quantity) | 销售金额 | 求和 |
| SUM(cost_price × quantity) | 成本金额 | JOIN quote_items 后求和 |
| 销售金额 - 成本金额 | 利润 | 计算列 |
| 利润 / 销售金额 | 利润率 | 计算列 |

---

## 四、UUID 生成规则

### 4.1 生成算法

```
uuid = sha256(goods_name.trim() + "|" + spec.trim()).hex()[0..16]
```

- 使用 SHA-256 对 `货物名称|规格型号` 做哈希
- 取十六进制字符串的前 16 位作为 UUID
- 同一物品（名称+规格相同）永远生成相同 UUID
- 规格为空时：`sha256(goods_name.trim() + "|")`

### 4.2 示例

```
货物名称: "螺栓"，规格型号: "M8×20"
→ sha256("螺栓|M8×20") = "a3f2c1d4e5b6a7f8..."
→ uuid = "a3f2c1d4e5b6a7f8"
```

---

## 五、关联逻辑

### 5.1 报价表导入流程

```
读取 Excel
  ↓
对每行提取 7 个字段
  ↓
生成 uuid = sha256(goods_name + "|" + spec)[0..16]
  ↓
查询 quote_items WHERE uuid = ?
  ├── 不存在 → INSERT 新记录
  └── 存在且价格有变化 → 
        写入 quote_price_history（保存旧价格）
        UPDATE quote_items（更新为新价格）
```

> 注意：导入为一次性覆盖模式（先 DELETE ALL，再批量 INSERT），仅用于从 Excel 迁移数据。历史价格表随之清空重建。后续如需增量更新，可改为 UPSERT 模式。

### 5.2 销售表导入流程

```
读取 Excel
  ↓
对每行提取 11 个字段
  ↓
自动匹配：
  生成 match_uuid = sha256(product_name.trim() + "|" + spec.trim())[0..16]
  查询 quote_items WHERE uuid = match_uuid
  ├── 找到 → item_uuid = match_uuid
  └── 未找到 → item_uuid = NULL（未关联，可后续手动处理）
  ↓
INSERT INTO sales_items
```

### 5.3 合同管理聚合查询

```sql
SELECT
    s.contract_no,
    MAX(s.customer)      AS customer,
    MAX(s.sale_date)     AS sale_date,
    MAX(s.project_name)  AS project_name,
    COUNT(*)             AS product_count,
    SUM(s.unit_price * s.quantity)                    AS total_sale_amount,
    SUM(COALESCE(q.cost_price, 0) * s.quantity)       AS total_cost_amount,
    SUM(s.unit_price * s.quantity)
      - SUM(COALESCE(q.cost_price, 0) * s.quantity)   AS total_profit
FROM sales_items s
LEFT JOIN quote_items q ON s.item_uuid = q.uuid
GROUP BY s.contract_no
ORDER BY s.sale_date DESC
```

> 未关联（`item_uuid IS NULL`）的行不参与成本和利润计算，使用 INNER JOIN 或在 SUM 中加 CASE 过滤。

### 5.4 未关联记录处理

销售明细中 `item_uuid = NULL` 的记录：
- 合同管理中该行**不参与利润计算**（成本金额和利润均排除此行）
- 前端标记为"未关联"，提示用户
- 后续可在销售明细中手动选择关联的报价物品（待实现）

---

## 六、模块职责边界

| 模块 | 数据来源 | 可写操作 | 只读操作 |
|------|---------|---------|---------|
| 报价明细 | `quote_items` | 导入（覆盖） | 查询、搜索、查看历史价格 |
| 销售明细 | `sales_items` | 导入（覆盖） | 查询、搜索 |
| 合同管理 | JOIN 聚合 | 无（后续扩展签收） | 按合同号聚合、利润计算 |
| 数据管理 | 两张主表 | 导入、清空 | 统计、预览 |

---

## 七、后续扩展预留

- **签收管理**：在 `sales_items` 中新增 `sign_person`、`sign_date`、`reconcile_date` 等字段，或单独建 `sign_records` 表
- **手动关联**：在销售明细界面提供"关联报价"操作，更新 `item_uuid`
- **价格修改**：合同管理中支持覆盖单行的 `unit_price`，写入 `sales_items`
- **多次报价**：若同一物品有多个供应商报价，可扩展 UUID 规则加入供应商维度
