import { ref, computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import { computeQuoteRow } from '../constants/salesColumns';
import type { QuoteRow } from '../constants/salesColumns';

/** 列表展示列 */
export const QUOTE_LIST_COLUMNS = [
  '报价序号',
  '区域',
  '日期',
  '客户的合同号',
  '货物名称',
  '规格型号',
  '单位',
  '数量',
  '供应商',
  '税率',
  '成本单价（含税）',
  '金额',
  '销售单价(含税)',
  '金额2',
  '利润',
  '(成本➗销售价)',
  '最后成交单价',
  '金额3',
  '单价差异',
] as const;

interface PageResult {
  rows: QuoteRow[];
  total: number;
  page: number;
  page_size: number;
}

export function useQuoteView() {
  const isLoading = ref(false);
  const rows      = ref<QuoteRow[]>([]);
  const total     = ref(0);
  const curPage   = ref(1);
  const pageSize  = 50;

  // 输入框绑定值（用户正在输入，未提交）
  const filterContractNo = ref('');
  const filterGoodsName  = ref('');
  const filterSpec       = ref('');
  const filterSupplier   = ref('');

  // 已提交的条件快照（翻页时使用，不受输入框实时变化影响）
  const committedConditions = ref<[string, string][]>([]);

  const toastMsg  = ref('');
  const toastType = ref<'success' | 'error'>('success');
  let toastTimer: ReturnType<typeof setTimeout> | null = null;

  const totalPages = computed(() => Math.max(1, Math.ceil(total.value / pageSize)));

  // 用已提交的快照去后端查询
  async function loadPage() {
    isLoading.value = true;
    try {
      const result = await invoke<PageResult>('query_quote_page', {
        page: curPage.value,
        pageSize,
        conditions: committedConditions.value,
      });
      rows.value = result.rows.map(r => computeQuoteRow(r));
      total.value = result.total;
    } catch (e) {
      showToast(`加载失败: ${e}`, 'error');
    } finally {
      isLoading.value = false;
    }
  }

  // 点击搜索：提交快照 → 回第1页 → 查询
  function onSearch() {
    const conds: [string, string][] = [];
    if (filterContractNo.value.trim()) conds.push(['客户的合同号', filterContractNo.value.trim()]);
    if (filterGoodsName.value.trim())  conds.push(['货物名称',     filterGoodsName.value.trim()]);
    if (filterSpec.value.trim())       conds.push(['规格型号',     filterSpec.value.trim()]);
    if (filterSupplier.value.trim())   conds.push(['供应商',       filterSupplier.value.trim()]);
    committedConditions.value = conds;
    curPage.value = 1;
    loadPage();
  }

  // 重置：清空输入框和快照
  function onReset() {
    filterContractNo.value = '';
    filterGoodsName.value  = '';
    filterSpec.value       = '';
    filterSupplier.value   = '';
    committedConditions.value = [];
    curPage.value = 1;
    loadPage();
  }

  // 翻页：直接用快照，不读输入框
  function prevPage() {
    if (curPage.value > 1) { curPage.value--; loadPage(); }
  }

  function nextPage() {
    if (curPage.value < totalPages.value) { curPage.value++; loadPage(); }
  }

  function showToast(msg: string, type: 'success' | 'error') {
    toastMsg.value = msg;
    toastType.value = type;
    if (toastTimer) clearTimeout(toastTimer);
    toastTimer = setTimeout(() => (toastMsg.value = ''), type === 'error' ? 5000 : 2500);
  }

  onMounted(loadPage);

  return {
    QUOTE_LIST_COLUMNS,
    isLoading,
    rows,
    total,
    curPage,
    pageSize,
    totalPages,
    filterContractNo,
    filterGoodsName,
    filterSpec,
    filterSupplier,
    toastMsg,
    toastType,
    onSearch,
    onReset,
    prevPage,
    nextPage,
  };
}
