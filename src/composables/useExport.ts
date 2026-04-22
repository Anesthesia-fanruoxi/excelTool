import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import * as XLSX from 'xlsx';
import { SALES_COLUMNS, computeRow } from '../constants/salesColumns';
import type { SalesRow } from '../constants/salesColumns';

interface PageResult {
  rows: SalesRow[];
  total: number;
  page: number;
  page_size: number;
}

const BATCH_SIZE = 1000;

export function useExport() {
  const isExporting = ref(false);
  const exportProgress = ref(0);

  /**
   * 分批拉取数据并导出为 xlsx
   * @param search      搜索关键词（空=全部）
   * @param statusFilter 状态筛选（空=全部）
   * @param filename    文件名（不含扩展名）
   */
  async function exportXlsx(search: string, statusFilter: string, filename: string) {
    isExporting.value = true;
    exportProgress.value = 0;

    try {
      // 第一批，同时获取总数
      const first = await invoke<PageResult>('query_sales_page', {
        page: 1,
        pageSize: BATCH_SIZE,
        search,
        statusFilter,
      });

      const total = first.total;
      if (total === 0) {
        isExporting.value = false;
        return;
      }

      const totalPages = Math.ceil(total / BATCH_SIZE);
      const allRows: SalesRow[] = [];

      first.rows.forEach((r) => allRows.push(computeRow(r)));
      exportProgress.value = Math.round((1 / totalPages) * 100);

      // 分批拉取剩余页
      for (let page = 2; page <= totalPages; page++) {
        const result = await invoke<PageResult>('query_sales_page', {
          page,
          pageSize: BATCH_SIZE,
          search,
          statusFilter,
        });
        result.rows.forEach((r) => allRows.push(computeRow(r)));
        exportProgress.value = Math.round((page / totalPages) * 100);
      }

      // 构建 worksheet 数据（表头 + 数据行）
      const headers = [...SALES_COLUMNS];
      const sheetData: string[][] = [
        headers,
        ...allRows.map((row) => headers.map((h) => row[h] ?? '')),
      ];

      const ws = XLSX.utils.aoa_to_sheet(sheetData);

      // 设置列宽（根据表头长度自动估算）
      ws['!cols'] = headers.map((h) => ({
        wch: Math.max(h.length * 2, 10),
      }));

      // 冻结首行
      ws['!freeze'] = { xSplit: 0, ySplit: 1 };

      const wb = XLSX.utils.book_new();
      XLSX.utils.book_append_sheet(wb, ws, '销售表');

      // 触发下载
      XLSX.writeFile(wb, `${filename}.xlsx`);
    } finally {
      isExporting.value = false;
      exportProgress.value = 0;
    }
  }

  return { isExporting, exportProgress, exportXlsx };
}
