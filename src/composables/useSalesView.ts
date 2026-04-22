import { ref, computed, onMounted, watch } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import {
  SALES_COLUMNS,
  LIST_COLUMNS,
  COMPUTED_COLUMNS,
  computeRow,
} from '../constants/salesColumns';
import type { SalesRow } from '../constants/salesColumns';

interface PageResult {
  rows: SalesRow[];
  total: number;
  page: number;
  page_size: number;
}

export function useSalesView() {
  const isLoading = ref(false);
  const isSaving  = ref(false);

  // 分页数据
  const rows      = ref<SalesRow[]>([]);
  const total     = ref(0);
  const curPage   = ref(1);
  const pageSize  = 50;

  // 搜索
  const searchText   = ref('');
  const searchCol    = ref('');

  // 详情弹框
  const detailRow = ref<SalesRow | null>(null);

  // 编辑弹框
  type DialogMode = 'add' | 'edit';
  const dialog = ref<{
    show: boolean;
    mode: DialogMode;
    rowId: number | null;
    form: SalesRow;
  }>({ show: false, mode: 'add', rowId: null, form: {} });

  // 删除确认
  const deleteConfirm = ref<{ show: boolean; rowId: number | null }>({
    show: false, rowId: null,
  });

  // toast
  const toastMsg  = ref('');
  const toastType = ref<'success' | 'error'>('success');
  let toastTimer: ReturnType<typeof setTimeout> | null = null;

  const totalPages = computed(() => Math.max(1, Math.ceil(total.value / pageSize)));

  // ── 加载当前页 ────────────────────────────────────────
  async function loadPage() {
    isLoading.value = true;
    try {
      const result = await invoke<PageResult>('query_sales_page', {
        page: curPage.value,
        pageSize,
        search: searchText.value,
        statusFilter: '',
      });
      // 注入计算列
      rows.value = result.rows.map((r) => computeRow(r));
      total.value = result.total;
    } catch (e) {
      showToast(`加载失败: ${e}`, 'error');
    } finally {
      isLoading.value = false;
    }
  }

  function onSearch() {
    curPage.value = 1;
    loadPage();
  }

  // ── 详情 ──────────────────────────────────────────────
  function openDetail(row: SalesRow) { detailRow.value = row; }
  function closeDetail() { detailRow.value = null; }

  // ── 新增 ──────────────────────────────────────────────
  function openAdd() {
    const form: SalesRow = {};
    SALES_COLUMNS.forEach((c) => { form[c] = ''; });
    dialog.value = { show: true, mode: 'add', rowId: null, form };
    startWatchForm();
  }

  // ── 编辑 ──────────────────────────────────────────────
  function openEdit(row: SalesRow) {
    const rowId = row['__id'] ? Number(row['__id']) : null;
    dialog.value = { show: true, mode: 'edit', rowId, form: { ...row } };
    detailRow.value = null;
    startWatchForm();
  }

  // ── 监听表单自动计算 ──────────────────────────────────
  let stopWatch: (() => void) | null = null;
  function startWatchForm() {
    if (stopWatch) stopWatch();
    stopWatch = watch(
      () => ({ ...dialog.value.form }),
      (newForm) => {
        const result = computeRow(newForm);
        COMPUTED_COLUMNS.forEach((col) => {
          if (dialog.value.form[col] !== result[col]) {
            dialog.value.form[col] = result[col];
          }
        });
      },
      { deep: true }
    );
  }

  // ── 保存 ──────────────────────────────────────────────
  async function saveDialog() {
    isSaving.value = true;
    const isAdd = dialog.value.mode === 'add';
    try {
      const finalRow = computeRow(dialog.value.form);
      // 去掉计算列和 __id 再存储
      const rowData: SalesRow = {};
      SALES_COLUMNS.forEach((c) => {
        if (!COMPUTED_COLUMNS.has(c)) rowData[c] = finalRow[c] ?? '';
      });

      await invoke('save_sales_row', {
        id: dialog.value.rowId ?? null,
        rowData,
      });

      if (stopWatch) { stopWatch(); stopWatch = null; }
      dialog.value.show = false;
      await loadPage();
      showToast(isAdd ? '新增成功' : '修改成功', 'success');
    } catch (e) {
      showToast(`保存失败: ${e}`, 'error');
    } finally {
      isSaving.value = false;
    }
  }

  // ── 删除 ──────────────────────────────────────────────
  function confirmDelete(row: SalesRow) {
    const rowId = row['__id'] ? Number(row['__id']) : null;
    deleteConfirm.value = { show: true, rowId };
  }

  async function doDelete() {
    if (deleteConfirm.value.rowId === null) return;
    try {
      await invoke('delete_sales_row', { id: deleteConfirm.value.rowId });
      deleteConfirm.value.show = false;
      detailRow.value = null;
      await loadPage();
      showToast('已删除', 'success');
    } catch (e) {
      showToast(`删除失败: ${e}`, 'error');
    }
  }

  // ── Toast ─────────────────────────────────────────────
  function showToast(msg: string, type: 'success' | 'error') {
    toastMsg.value = msg;
    toastType.value = type;
    if (toastTimer) clearTimeout(toastTimer);
    toastTimer = setTimeout(() => (toastMsg.value = ''), type === 'error' ? 5000 : 2500);
  }

  onMounted(loadPage);

  return {
    LIST_COLUMNS,
    SALES_COLUMNS,
    COMPUTED_COLUMNS,
    isLoading,
    isSaving,
    rows,
    total,
    searchText,
    searchCol,
    curPage,
    pageSize,
    totalPages,
    detailRow,
    dialog,
    deleteConfirm,
    toastMsg,
    toastType,
    onSearch,
    loadPage,
    openDetail,
    closeDetail,
    openAdd,
    openEdit,
    saveDialog,
    confirmDelete,
    doDelete,
  };
}
