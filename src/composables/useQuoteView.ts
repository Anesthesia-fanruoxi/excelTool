import { ref, computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import {
  QUOTE_LIST_FIELDS, QUOTE_FILTER_FIELDS, QUOTE_FIELD_LABELS,
  type QuoteItem,
} from '../constants/columns';

interface PageResult { rows: QuoteItem[]; total: number; page: number; page_size: number }

export function useQuoteView() {
  const isLoading          = ref(false);
  const rows               = ref<QuoteItem[]>([]);
  const total              = ref(0);
  const curPage            = ref(1);
  const pageSize           = 50;
  const editingItem        = ref<QuoteItem | null>(null);
  const showCreateContract = ref(false);

  // 跨页多选：用普通数组，响应式最可靠
  const selectedItems = ref<QuoteItem[]>([]);
  const selectedCount = computed(() => selectedItems.value.length);

  const filterGoodsName = ref('');
  const filterSpec      = ref('');
  const filterSupplier  = ref('');
  const filterDate      = ref('');
  const committed       = ref<[string, string][]>([]);

  const toastMsg  = ref('');
  const toastType = ref<'success' | 'error'>('success');
  let toastTimer: ReturnType<typeof setTimeout> | null = null;

  const totalPages = computed(() => Math.max(1, Math.ceil(total.value / pageSize)));

  async function loadPage() {
    isLoading.value = true;
    try {
      const result = await invoke<PageResult>('query_quote_page', {
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

  function toggleSelect(item: QuoteItem) {
    const idx = selectedItems.value.findIndex(s => s.uuid === item.uuid);
    if (idx >= 0) {
      selectedItems.value.splice(idx, 1);
    } else {
      selectedItems.value.push({ ...item });
    }
  }

  function isSelected(uuid: string) {
    return selectedItems.value.some(s => s.uuid === uuid);
  }

  function clearSelection() {
    selectedItems.value = [];
  }

  function onSearch() {
    const conds: [string, string][] = [];
    if (filterGoodsName.value.trim()) conds.push(['goods_name', filterGoodsName.value.trim()]);
    if (filterSpec.value.trim())      conds.push(['spec',       filterSpec.value.trim()]);
    if (filterSupplier.value.trim())  conds.push(['supplier',   filterSupplier.value.trim()]);
    if (filterDate.value.trim())      conds.push(['date',       filterDate.value.trim()]);
    committed.value = conds;
    curPage.value   = 1;
    loadPage();
  }

  function onReset() {
    filterGoodsName.value = '';
    filterSpec.value      = '';
    filterSupplier.value  = '';
    filterDate.value      = '';
    committed.value       = [];
    curPage.value         = 1;
    loadPage();
  }

  function prevPage() { if (curPage.value > 1) { curPage.value--; loadPage(); } }
  function nextPage() { if (curPage.value < totalPages.value) { curPage.value++; loadPage(); } }

  function openEdit(item: QuoteItem) { editingItem.value = item; }
  function closeEdit() { editingItem.value = null; }
  function onSaved() { editingItem.value = null; showToast('价格已更新', 'success'); loadPage(); }

  function openCreateContract() {
    if (selectedCount.value === 0) { showToast('请先勾选报价物品', 'error'); return; }
    showCreateContract.value = true;
  }

  function onContractSaved() {
    showCreateContract.value = false;
    clearSelection();
    showToast('合同创建成功，可在合同管理中查看', 'success');
  }

  function showToast(msg: string, type: 'success' | 'error') {
    toastMsg.value  = msg;
    toastType.value = type;
    if (toastTimer) clearTimeout(toastTimer);
    toastTimer = setTimeout(() => (toastMsg.value = ''), type === 'error' ? 5000 : 2500);
  }

  onMounted(loadPage);

  return {
    QUOTE_LIST_FIELDS, QUOTE_FILTER_FIELDS, QUOTE_FIELD_LABELS,
    isLoading, rows, total, curPage, pageSize, totalPages,
    filterGoodsName, filterSpec, filterSupplier, filterDate, editingItem,
    selectedItems, selectedCount, showCreateContract,
    toastMsg, toastType,
    onSearch, onReset, prevPage, nextPage,
    openEdit, closeEdit, onSaved,
    toggleSelect, isSelected, clearSelection,
    openCreateContract, onContractSaved,
  };
}
