export interface SheetInfo {
  name: string;
  index: number;
  row_count: number;
  col_count: number;
}

export interface ExcelData {
  sheets: SheetInfo[];
  file_path: string;
  file_name: string;
}

export interface TabData {
  id: string;
  fileName: string;
  filePath: string;
  sheetName: string;
  sheetIndex: number;
  tableName: string;
  headers: string[];  // 完整表头（含公式列标记 [公式]）
}

export interface PageResult {
  columns: string[];
  rows: (string | null)[][];
  total: number;
  page: number;
  page_size: number;
}
