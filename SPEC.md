# Excel 工具 - 项目规范

## 1. 项目概述

- **项目名称**: excel-tool
- **类型**: Tauri 桌面应用 (Vue3 前端 + Rust 后端)
- **核心功能**: 读取Excel文件、展示数据、支持导入到本地加密库
- **目标用户**: 需要处理Excel数据的本地用户

## 2. 技术栈

| 层级 | 技术 | 用途 |
|------|------|------|
| 前端 | Vue3 + Vite | UI展示 |
| 桌面框架 | Tauri | 桌面应用封装 |
| Excel解析 | calamine (Rust) | 读取 .xlsx/.xls |
| 加密 | ring (Rust) | AES加密 |
| 机器码 | hardware-uuid (Rust) | 获取唯一机器标识 |

## 3. 功能模块

### 3.1 主菜单
- [ ] 打开Excel文件
- [ ] 查看最近文件
- [ ] 导入到库
- [ ] 导出数据
- [ ] 设置

### 3.2 Excel读取
- 支持 .xlsx 和 .xls 格式
- 读取工作表列表
- 按工作表展示数据
- 支持大数据量分页

### 3.3 本地加密库
- 数据存储格式: AES-256-GCM 加密的 JSON 文件
- 加密盐: 基于机器码生成（CPU序列号/主板UUID等）
- 文件位置: `~/.excel-tool/vault.enc`

### 3.4 数据展示
- 表格视图（类似Excel预览）
- 支持筛选、排序
- 分页加载

## 4. 项目结构

```
excel-tool/
├── src/                    # Vue前端
│   ├── components/
│   │   ├── MenuBar.vue     # 菜单栏
│   │   ├── ExcelViewer.vue # Excel数据展示
│   │   └── VaultManager.vue# 加密库管理
│   ├── views/
│   │   └── Home.vue
│   ├── stores/             # Pinia状态
│   └── App.vue
├── src-tauri/              # Rust后端
│   ├── src/
│   │   ├── main.rs         # 入口
│   │   ├── commands.rs     # Tauri命令
│   │   ├── excel.rs        # calamine封装
│   │   ├── crypto.rs       # 加密模块
│   │   └── machine_id.rs   # 机器码获取
│   ├── Cargo.toml
│   └── tauri.conf.json
└── package.json
```

## 5. 加密设计

```
机器码 → PBKDF2(盐) → 派生密钥 → AES-256-GCM加密
```

- **机器码来源**: Windows注册表 / Mac sysctl
- **密钥派生**: PBKDF2-SHA256, 100000 iterations
- **存储格式**: base64(nonce || ciphertext || tag)

## 6. 菜单设计

```
文件
├── 打开 Excel...      (Ctrl+O)
├── 打开最近 >
│   ├── file1.xlsx
│   └── file2.xlsx
├── ─────────────
└── 退出              (Alt+F4)

数据
├── 导入到加密库      (Ctrl+I)
├── 从库导出...
└── 清空库

视图
├── 刷新              (F5)
└── 全屏              (F11)

帮助
├── 关于
└── 使用文档
```

## 7. 命令设计 (Rust → JS)

```rust
// Excel操作
#[tauri::command] fn open_excel(path: String) -> Result<ExcelData, String>
#[tauri::command] fn get_sheets(path: String) -> Result<Vec<SheetInfo>, String>

// 加密库
#[tauri::command] fn import_to_vault(data: Vec<Row>) -> Result<(), String>
#[tauri::command] fn export_from_vault() -> Result<Vec<Row>, String>

// 系统
#[tauri::command] fn get_machine_id() -> Result<String, String>
```

## 8. 验收标准

1. ✅ 可打开 .xlsx 文件并正确解析
2. ✅ 数据以表格形式展示
3. ✅ 可导入数据到加密库
4. ✅ 加密库仅本机可解密（机器码绑定）
5. ✅ 菜单功能完整
