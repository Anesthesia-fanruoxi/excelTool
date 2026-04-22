import { ref, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import { open } from '@tauri-apps/api/dialog';
import { SALES_COLUMNS, COMPUTED_COLUMNS } from '../constants/salesColumns';
import type { ExcelData, VaultEntry } from '../types';

interface TableStats {
  name: string;
  count: number;
}

interface ConfirmDialog {
  show: boolean;
  title: string;
  message: string;
  onConfirm: () => void;
}

/** 列名标准化：去空格、统一半角括号为全角 */
function normalizeCol(col: string): string {
  return col
    .replace(/\n/g, '')
    .replace(/[^\u4e00-\u9fa5a-zA-Z0-9\s]/g, '')
    .replace(/\s+/g, ' ')
    .trim();
}

function validateHeaders(headers: string[]): string | null {
  const normalized = headers.map(normalizeCol);
  const missing = SALES_COLUMNS.filter(
    (col) => !normalized.includes(normalizeCol(col))
  );
  return missing.length > 0
    ? `列不匹配，缺少：${missing.join('、')}`
    : null;
}

export function useDataManager() {
  const FIXED_TABLES = [
    { key: 'sales', label: '销售表', icon: '📊', desc: '销售订单数据' },
  ];

  const isLoading = ref(false);
  const isImporting = ref(false);
  const tableStats = ref<Record<string, number>>({ sales: 0 });
  const flippedCard = ref<string | null>(null);

  const confirmDialog = ref<ConfirmDialog>({
    show: false,
    title: '',
    message: '',
    onConfirm: () => {},
  });

  const previewEntry = ref<VaultEntry | null>(null);
  const previewPage = ref(1);
  const previewPageSize = 50;

  const toastMsg = ref('');
  const toastType = ref<'success' | 'error'>('success');
  let toastTimer: ReturnType<typeof setTimeout> | null = null;

  // 加载统计
  async function loadStats() {
    isLoading.value = true;
    try {
      const stats = await invoke<TableStats>('get_table_stats');
      tableStats.value.sales = stats.count;
    } catch (e) {
      console.error('加载统计失败:', e);
    } finally {
      isLoading.value = false;
    }
  }

  // 翻转卡片
  function flipCard(key: string) {
    flippedCard.value = flippedCard.value === key ? null : key;
  }

  // 预览
  async function handlePreview() {
    try {
      const data = await invoke<{ entries: VaultEntry[] }>('get_vault_status');
      const entry = data.entries.find((e) => e.name === '销售表');
      if (!entry) {
        showToast('暂无数据', 'error');
        return;
      }
      previewEntry.value = entry;
      previewPage.value = 1;
      flippedCard.value = null;
    } catch (e) {
      showToast(`加载失败: ${e}`, 'error');
    }
  }

  // 清空
  function handleClear() {
    showConfirm(
      '⚠️ 清空销售表',
      '此操作将永久删除销售表中的所有数据，且无法恢复。确认继续？',
      async () => {
        try {
          await invoke('clear_sales_table');
          tableStats.value.sales = 0;
          showToast('销售表已清空', 'success');
          flippedCard.value = null;
        } catch (e) {
          showToast(`清空失败: ${e}`, 'error');
        }
      }
    );
  }

  // 导入
  function handleImport() {
    showConfirm(
      '⚠️ 导入数据',
      '导入将覆盖销售表中的现有数据，原有数据将被清空。确认继续？',
      () => doImport()
    );
  }

  async function doImport() {
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: 'Excel 文件', extensions: ['xlsx', 'xls', 'xlsm'] }],
      });
      if (!selected) return;

      const path = selected as string;
      isImporting.value = true;

      const excelData = await invoke<ExcelData>('open_excel', { path });
      if (!excelData.sheets.length) {
        showToast('Excel 文件中没有工作表', 'error');
        return;
      }

      const sheetData = await invoke<string[][]>('read_sheet_data', {
        path,
        sheetIndex: 0,
      });

      if (sheetData.length < 2) {
        showToast('工作表数据为空', 'error');
        return;
      }

      const headers = sheetData[0].map((h) =>
        h
          .replace(/\n/g, '')
          .replace(/[^\u4e00-\u9fa5a-zA-Z0-9\s]/g, '')
          .replace(/\s+/g, ' ')
          .trim()
      );
      const colError = validateHeaders(headers);
      if (colError) {
        showToast(colError, 'error');
        return;
      }

      // 过滤掉计算列，只存储原始数据列
      const rawHeaders = headers.filter((h) => !COMPUTED_COLUMNS.has(h.trim()));
      const headerIndexes = rawHeaders.map((h) => headers.indexOf(h));
      const data = sheetData.slice(1).map((row) =>
        headerIndexes.map((i) => row[i] ?? '')
      );

      await invoke('import_sales', {
        headers: rawHeaders,
        data,
      });
      await invoke('add_recent_file', { filePath: path });

      tableStats.value.sales = data.length;
      showToast(`导入成功，共 ${data.length} 条数据`, 'success');
      flippedCard.value = null;
    } catch (e) {
      showToast(`导入失败: ${e}`, 'error');
    } finally {
      isImporting.value = false;
    }
  }

  // 确认弹框
  function showConfirm(title: string, message: string, onConfirm: () => void) {
    confirmDialog.value = { show: true, title, message, onConfirm };
  }

  function confirmAction() {
    confirmDialog.value.onConfirm();
    confirmDialog.value.show = false;
  }

  function cancelConfirm() {
    confirmDialog.value.show = false;
  }

  // Toast
  function showToast(msg: string, type: 'success' | 'error') {
    toastMsg.value = msg;
    toastType.value = type;
    if (toastTimer) clearTimeout(toastTimer);
    toastTimer = setTimeout(
      () => (toastMsg.value = ''),
      type === 'error' ? 5000 : 3000
    );
  }

  // 预览分页
  function getPreviewRows() {
    if (!previewEntry.value) return [];
    const start = (previewPage.value - 1) * previewPageSize;
    return previewEntry.value.data.slice(start, start + previewPageSize);
  }

  function previewTotalPages() {
    if (!previewEntry.value) return 1;
    return Math.max(
      1,
      Math.ceil(previewEntry.value.data.length / previewPageSize)
    );
  }

  // ESC
  function handleKeydown(e: KeyboardEvent) {
    if (e.key !== 'Escape') return;
    if (previewEntry.value) { previewEntry.value = null; return; }
    if (confirmDialog.value.show) { cancelConfirm(); return; }
    if (flippedCard.value) { flippedCard.value = null; }
  }

  onMounted(() => {
    loadStats();
    window.addEventListener('keydown', handleKeydown);
  });

  onUnmounted(() => {
    window.removeEventListener('keydown', handleKeydown);
    if (toastTimer) clearTimeout(toastTimer);
  });

  return {
    FIXED_TABLES,
    isLoading,
    isImporting,
    tableStats,
    flippedCard,
    confirmDialog,
    previewEntry,
    previewPage,
    previewPageSize,
    toastMsg,
    toastType,
    flipCard,
    handlePreview,
    handleClear,
    handleImport,
    confirmAction,
    cancelConfirm,
    getPreviewRows,
    previewTotalPages,
  };
}
