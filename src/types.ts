export interface SheetInfo {
  name: string;
  index: number;
  row_count: number;
  col_count: number;
}

export interface ExcelData {
  sheets: SheetInfo[];
  file_path: string;
}

export interface VaultEntry {
  id: string;
  name: string;
  data: string[][];
  headers: string[];
  imported_at: string;
  source_file: string;
}

export interface VaultData {
  entries: VaultEntry[];
}
