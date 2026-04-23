import { ref, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import { open } from '@tauri-apps/api/dialog';
import type { ExcelData } from '../types';

interface TableStats { name: string; count: number }
interface ConfirmDialog { show: boolean; title: string; message: string; onConfirm: () => void }
interface PreviewData { headers: string[]; rows: string[][]; total: number }

export function useDataManager() {
  const FIXED_TABLES = [
    { key: 'quote', label: '报价表', icon: '💰', desc: '供应商报价物品数据' },
    { key: 'sales', label: '销售表', icon: '📊', desc: '客户销售明细数据' },
  ];

  const isLoading   = ref(false);
  const isImporting = ref(false);
  const tableStats  = ref<Record<string, number>>({ quote: 0, sales: 0 });
  const flippedCard = ref<string | null>(null);

  const confirmDialog = ref<ConfirmDialog>({ show: false, title: '', message: '', onConfirm: () => {} });

  const previewData    = ref<PreviewData | null>(null);
  const previewLabel   = ref('');
  const previewKey     = ref('');
  const previewPage    = ref(1);
  const previewPageSize = 50;

  const toastMsg  = ref('');
  const toastType = ref<'success' | 'error'>('success');
  let toastTimer: ReturnType<typeof setTimeout> | null = null;

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

  function flipCard(key: string) {
    flippedCard.value = flippedCard.value === key ? null : key;
  }

  async function handlePreview(key: string) {
    try {
      const cmd = key === 'quote' ? 'preview_quote_data' : 'preview_sales_data';
      const data = await invoke<PreviewData>(cmd, { page: 1, pageSize: previewPageSize });
      previewData.value  = data;
      previewLabel.value = key === 'quote' ? '报价表' : '销售表';
      previewKey.value   = key;
      previewPage.value  = 1;
      flippedCard.value  = null;
    } catch (e) {
      showToast(`加载失败: ${e}`, 'error');
    }
  }

  async function loadPreviewPage(page: number) {
    if (!previewKey.value) return;
    try {
      const cmd = previewKey.value === 'quote' ? 'preview_quote_data' : 'preview_sales_data';
      const data = await invoke<PreviewData>(cmd, { page, pageSize: previewPageSize });
      previewData.value = data;
      previewPage.value = page;
    } catch (e) {
      showToast(`加载失败: ${e}`, 'error');
    }
  }

  function previewTotalPages() {
    if (!previewData.value) return 1;
    return Math.max(1, Math.ceil(previewData.value.total / previewPageSize));
  }

  function handleClear(key: string) {
    const label = key === 'quote' ? '报价表' : '销售表';
    const cmd   = key === 'quote' ? 'clear_quote_table' : 'clear_sales_table';
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

  function handleImport(key: string) {
    const label = key === 'quote' ? '报价表' : '销售表';
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
      if (!excelData.sheets.length) { showToast('Excel 文件中没有工作表', 'error'); return; }

      const sheetData = await invoke<string[][]>('read_sheet_data', { path, sheetIndex: 0 });
      if (sheetData.length < 2) { showToast('工作表数据为空', 'error'); return; }

      // 原始表头（不做任何过滤，交给后端映射）
      const headers = sheetData[0].map(h => h.replace(/\n/g, '').replace(/\r/g, '').trim());
      const data    = sheetData.slice(1);

      if (key === 'quote') {
        const count = await invoke<number>('import_quote_items', { headers, data });
        tableStats.value.quote = count;
        showToast(`报价表导入成功，共 ${count} 条物品`, 'success');
      } else {
        const result = await invoke<{ total: number; linked: number; unlinked: number }>(
          'import_sales_items', { headers, data }
        );
        tableStats.value.sales = result.total;
        const tip = result.unlinked > 0
          ? `销售表导入成功，共 ${result.total} 条，其中 ${result.linked} 条已关联报价，${result.unlinked} 条未匹配`
          : `销售表导入成功，共 ${result.total} 条，全部已关联报价`;
        showToast(tip, result.unlinked > 0 ? 'error' : 'success');
      }

      await invoke('add_recent_file', { filePath: path });
      flippedCard.value = null;
    } catch (e) {
      showToast(`导入失败: ${e}`, 'error');
    } finally {
      isImporting.value = false;
    }
  }

  function showConfirm(title: string, message: string, onConfirm: () => void) {
    confirmDialog.value = { show: true, title, message, onConfirm };
  }
  function confirmAction() { confirmDialog.value.onConfirm(); confirmDialog.value.show = false; }
  function cancelConfirm() { confirmDialog.value.show = false; }

  function showToast(msg: string, type: 'success' | 'error') {
    toastMsg.value  = msg;
    toastType.value = type;
    if (toastTimer) clearTimeout(toastTimer);
    toastTimer = setTimeout(() => (toastMsg.value = ''), type === 'error' ? 6000 : 3000);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key !== 'Escape') return;
    if (previewData.value) { previewData.value = null; return; }
    if (confirmDialog.value.show) { cancelConfirm(); return; }
    if (flippedCard.value) { flippedCard.value = null; }
  }

  onMounted(() => { loadStats(); window.addEventListener('keydown', handleKeydown); });
  onUnmounted(() => { window.removeEventListener('keydown', handleKeydown); if (toastTimer) clearTimeout(toastTimer); });

  return {
    FIXED_TABLES, isLoading, isImporting, tableStats, flippedCard,
    confirmDialog, previewData, previewLabel, previewPage, previewPageSize,
    toastMsg, toastType,
    flipCard, handlePreview, handleClear, handleImport,
    confirmAction, cancelConfirm, loadPreviewPage, previewTotalPages,
  };
}
