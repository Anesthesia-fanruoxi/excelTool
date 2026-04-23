import { ref, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import { open } from '@tauri-apps/api/dialog';
import { SALES_COLUMNS, COMPUTED_COLUMNS, QUOTE_COLUMNS, QUOTE_COMPUTED_COLUMNS } from '../constants/salesColumns';
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

/** 列名标准化：去符号、去空格 */
function normalizeCol(col: string): string {
  return col
    .replace(/\n/g, '')
    .replace(/[^\u4e00-\u9fa5a-zA-Z0-9\s➗（）()]/g, '')
    .replace(/\s+/g, ' ')
    .trim();
}

function validateHeaders(headers: string[], required: readonly string[]): string | null {
  const normalized = headers.map(normalizeCol);
  const missing = required.filter((col) => !normalized.includes(normalizeCol(col)));
  return missing.length > 0 ? `列不匹配，缺少：${missing.join('、')}` : null;
}

export function useDataManager() {
  const FIXED_TABLES = [
    { key: 'sales', label: '销售表', icon: '📊', desc: '销售订单数据' },
    { key: 'quote', label: '报价表', icon: '💰', desc: '报价单数据' },
  ];

  const isLoading = ref(false);
  const isImporting = ref(false);
  const tableStats = ref<Record<string, number>>({ sales: 0, quote: 0 });
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
      const [salesStats, quoteStats] = await Promise.all([
        invoke<TableStats>('get_table_stats'),
        invoke<TableStats>('get_quote_stats'),
      ]);
      tableStats.value.sales = salesStats.count;
      tableStats.value.quote = quoteStats.count;
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
  async function handlePreview(key: string) {
    try {
      if (key === 'sales') {
        const data = await invoke<{ entries: VaultEntry[] }>('get_vault_status');
        const entry = data.entries.find((e) => e.name === '销售表');
        if (!entry) { showToast('暂无数据', 'error'); return; }
        previewEntry.value = entry;
      } else {
        const data = await invoke<{ entries: VaultEntry[] }>('get_quote_vault_status');
        const entry = data.entries.find((e) => e.name === '报价表');
        if (!entry) { showToast('暂无数据', 'error'); return; }
        previewEntry.value = entry;
      }
      previewPage.value = 1;
      flippedCard.value = null;
    } catch (e) {
      showToast(`加载失败: ${e}`, 'error');
    }
  }

  // 清空
  function handleClear(key: string) {
    const label = key === 'sales' ? '销售表' : '报价表';
    const cmd   = key === 'sales' ? 'clear_sales_table' : 'clear_quote_table';
    showConfirm(
      `⚠️ 清空${label}`,
      `此操作将永久删除${label}中的所有数据，且无法恢复。确认继续？`,
      async () => {
        try {
          await invoke(cmd);
          tableStats.value[key] = 0;
          showToast(`${label}已清空`, 'success');
          flippedCard.value = null;
        } catch (e) {
          showToast(`清空失败: ${e}`, 'error');
        }
      }
    );
  }

  // 导入
  function handleImport(key: string) {
    const label = key === 'sales' ? '销售表' : '报价表';
    showConfirm(
      '⚠️ 导入数据',
      `导入将覆盖${label}中的现有数据，原有数据将被清空。确认继续？`,
      () => doImport(key)
    );
  }

  async function doImport(key: string) {
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

      const sheetData = await invoke<string[][]>('read_sheet_data', { path, sheetIndex: 0 });
      if (sheetData.length < 2) {
        showToast('工作表数据为空', 'error');
        return;
      }

      const headers = sheetData[0].map((h) =>
        h.replace(/\n/g, '').replace(/[^\u4e00-\u9fa5a-zA-Z0-9\s➗（）()]/g, '').replace(/\s+/g, ' ').trim()
      );

      if (key === 'sales') {
        await importSales(path, headers, sheetData);
      } else {
        await importQuote(path, headers, sheetData);
      }
    } catch (e) {
      showToast(`导入失败: ${e}`, 'error');
    } finally {
      isImporting.value = false;
    }
  }

  async function importSales(path: string, headers: string[], sheetData: string[][]) {
    const colError = validateHeaders(headers, SALES_COLUMNS);
    if (colError) { showToast(colError, 'error'); return; }

    const rawHeaders = headers.filter((h) => !COMPUTED_COLUMNS.has(h.trim()));
    const headerIndexes = rawHeaders.map((h) => headers.indexOf(h));
    const data = sheetData.slice(1).map((row) => headerIndexes.map((i) => row[i] ?? ''));

    await invoke('import_sales', { headers: rawHeaders, data });
    await invoke('add_recent_file', { filePath: path });
    tableStats.value.sales = data.length;
    showToast(`导入成功，共 ${data.length} 条数据`, 'success');
    flippedCard.value = null;
  }

  async function importQuote(path: string, headers: string[], sheetData: string[][]) {
    const colError = validateHeaders(headers, QUOTE_COLUMNS);
    if (colError) { showToast(colError, 'error'); return; }

    // 过滤掉计算列，只存储原始数据列
    const rawHeaders = headers.filter((h) => !QUOTE_COMPUTED_COLUMNS.has(h.trim()));
    const headerIndexes = rawHeaders.map((h) => headers.indexOf(h));
    const data = sheetData.slice(1).map((row) => headerIndexes.map((i) => row[i] ?? ''));

    await invoke('import_quote', { headers: rawHeaders, data });
    await invoke('add_recent_file', { filePath: path });
    tableStats.value.quote = data.length;
    showToast(`导入成功，共 ${data.length} 条数据`, 'success');
    flippedCard.value = null;
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
    toastTimer = setTimeout(() => (toastMsg.value = ''), type === 'error' ? 5000 : 3000);
  }

  // 预览分页
  function getPreviewRows() {
    if (!previewEntry.value) return [];
    const start = (previewPage.value - 1) * previewPageSize;
    return previewEntry.value.data.slice(start, start + previewPageSize);
  }

  function previewTotalPages() {
    if (!previewEntry.value) return 1;
    return Math.max(1, Math.ceil(previewEntry.value.data.length / previewPageSize));
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
