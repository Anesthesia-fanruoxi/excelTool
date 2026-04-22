import { ref, computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';

export interface ContractSummary {
  contract_no: string;
  customer: string;
  sale_date: string;
  project_name: string;
  row_count: number;
  total_amount: number;
  total_profit: number;
  reconcile_status: string;
}

export function useContractView() {
  const isLoading  = ref(false);
  const contracts  = ref<ContractSummary[]>([]);
  const statusFilter = ref('');
  const curPage    = ref(1);
  const pageSize   = 50;

  // 四个独立筛选条件（非实时，点搜索才生效）
  const filterContractNo  = ref('');
  const filterSaleDate    = ref('');
  const filterProjectName = ref('');
  const filterCustomer    = ref('');

  // 已应用的筛选条件快照
  const appliedContractNo  = ref('');
  const appliedSaleDate    = ref('');
  const appliedProjectName = ref('');
  const appliedCustomer    = ref('');

  // 展开的合同号
  const expandedContract = ref<string | null>(null);
  // 展开的明细数据
  const expandedRows = ref<Record<string, any[]>>({});

  const toastMsg  = ref('');
  const toastType = ref<'success' | 'error'>('success');
  let toastTimer: ReturnType<typeof setTimeout> | null = null;

  async function loadData() {
    isLoading.value = true;
    try {
      contracts.value = await invoke<ContractSummary[]>('query_contracts', {
        statusFilter: statusFilter.value,
      });
    } catch (e) {
      showToast(`加载失败: ${e}`, 'error');
    } finally {
      isLoading.value = false;
    }
  }

  // 状态排序权重：等回签 > 回签不完整 > 待对账 > 已对账 > 其他
  const STATUS_ORDER: Record<string, number> = {
    '等回签':    0,
    '回签不完整': 1,
    '待对账':    2,
    '已对账':    3,
  };

  function statusWeight(s: string): number {
    return STATUS_ORDER[s] ?? 99;
  }

  const filteredContracts = computed(() => {
    let list = contracts.value;

    if (appliedContractNo.value) {
      const kw = appliedContractNo.value.toLowerCase();
      list = list.filter(c => c.contract_no.toLowerCase().includes(kw));
    }
    if (appliedSaleDate.value) {
      const kw = appliedSaleDate.value.toLowerCase();
      list = list.filter(c => c.sale_date.toLowerCase().includes(kw));
    }
    if (appliedProjectName.value) {
      const kw = appliedProjectName.value.toLowerCase();
      list = list.filter(c => c.project_name.toLowerCase().includes(kw));
    }
    if (appliedCustomer.value) {
      const kw = appliedCustomer.value.toLowerCase();
      list = list.filter(c => c.customer.toLowerCase().includes(kw));
    }

    // 按签收状态排序
    return [...list].sort((a, b) =>
      statusWeight(a.reconcile_status) - statusWeight(b.reconcile_status)
    );
  });

  const totalPages = computed(() =>
    Math.max(1, Math.ceil(filteredContracts.value.length / pageSize))
  );

  const pagedContracts = computed(() => {
    const s = (curPage.value - 1) * pageSize;
    return filteredContracts.value.slice(s, s + pageSize);
  });

  function onSearch() {
    // 将输入框的值快照到 applied，触发过滤
    appliedContractNo.value  = filterContractNo.value;
    appliedSaleDate.value    = filterSaleDate.value;
    appliedProjectName.value = filterProjectName.value;
    appliedCustomer.value    = filterCustomer.value;
    curPage.value = 1;
  }

  function onReset() {
    filterContractNo.value  = '';
    filterSaleDate.value    = '';
    filterProjectName.value = '';
    filterCustomer.value    = '';
    appliedContractNo.value  = '';
    appliedSaleDate.value    = '';
    appliedProjectName.value = '';
    appliedCustomer.value    = '';
    curPage.value = 1;
  }

  async function toggleExpand(contractNo: string) {
    if (expandedContract.value === contractNo) {
      expandedContract.value = null;
      return;
    }
    expandedContract.value = contractNo;
    if (!expandedRows.value[contractNo]) {
      try {
        expandedRows.value[contractNo] = await invoke<any[]>('query_contract_detail', { contractNo });
      } catch (e) {
        showToast(`加载明细失败: ${e}`, 'error');
      }
    }
  }

  function showToast(msg: string, type: 'success' | 'error') {
    toastMsg.value = msg;
    toastType.value = type;
    if (toastTimer) clearTimeout(toastTimer);
    toastTimer = setTimeout(() => (toastMsg.value = ''), type === 'error' ? 5000 : 2500);
  }

  onMounted(loadData);

  return {
    isLoading,
    contracts,
    filterContractNo,
    filterSaleDate,
    filterProjectName,
    filterCustomer,
    statusFilter,
    curPage,
    pageSize,
    totalPages,
    pagedContracts,
    filteredContracts,
    expandedContract,
    expandedRows,
    toastMsg,
    toastType,
    onSearch,
    onReset,
    loadData,
    toggleExpand,
  };
}
