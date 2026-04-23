import { ref, computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import {
  SALES_LIST_FIELDS, SALES_FILTER_FIELDS, SALES_FIELD_LABELS,
  type SalesItem,
} from '../constants/columns';

interface PageResult { rows: SalesItem[]; total: number; page: number; page_size: number }

export function useSalesView() {
  const isLoading = ref(false);
  const rows      = ref<SalesItem[]>([]);
  const total     = ref(0);
  const curPage   = ref(1);
  const pageSize  = 50;

  const filterField = ref('contract_no');
  const filterKw    = ref('');
  const committed   = ref<[string, string][]>([]);

  const toastMsg  = ref('');
  const toastType = ref<'success' | 'error'>('success');
  let toastTimer: ReturnType<typeof setTimeout> | null = null;

  const totalPages = computed(() => Math.max(1, Math.ceil(total.value / pageSize)));

  async function loadPage() {
    isLoading.value = true;
    try {
      const result = await invoke<PageResult>('query_sales_page', {
        page: curPage.value,
        pageSize,
        conditions: committed.value,
      });
      rows.value  = result.rows;
      total.value = result.total;
    } catch (e) {
      showToast(`加载失败: ${e}`, 'error');
    } finally {
      isLoading.value = false;
    }
  }

  function onSearch() {
    const conds: [string, string][] = [];
    if (filterKw.value.trim()) conds.push([filterField.value, filterKw.value.trim()]);
    committed.value = conds;
    curPage.value   = 1;
    loadPage();
  }

  function onReset() {
    filterField.value = 'contract_no';
    filterKw.value    = '';
    committed.value   = [];
    curPage.value     = 1;
    loadPage();
  }

  function prevPage() { if (curPage.value > 1) { curPage.value--; loadPage(); } }
  function nextPage() { if (curPage.value < totalPages.value) { curPage.value++; loadPage(); } }

  function showToast(msg: string, type: 'success' | 'error') {
    toastMsg.value  = msg;
    toastType.value = type;
    if (toastTimer) clearTimeout(toastTimer);
    toastTimer = setTimeout(() => (toastMsg.value = ''), type === 'error' ? 5000 : 2500);
  }

  onMounted(loadPage);

  return {
    SALES_LIST_FIELDS, SALES_FILTER_FIELDS, SALES_FIELD_LABELS,
    isLoading, rows, total, curPage, pageSize, totalPages,
    filterField, filterKw,
    toastMsg, toastType,
    onSearch, onReset, prevPage, nextPage,
  };
}
