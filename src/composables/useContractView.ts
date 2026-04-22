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
  const searchText = ref('');
  const statusFilter = ref('');
  const curPage    = ref(1);
  const pageSize   = 50;

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

  const filteredContracts = computed(() => {
    if (!searchText.value) return contracts.value;
    const kw = searchText.value.toLowerCase();
    return contracts.value.filter((c) =>
      c.contract_no.toLowerCase().includes(kw) ||
      c.customer.toLowerCase().includes(kw) ||
      c.project_name.toLowerCase().includes(kw)
    );
  });

  const totalPages = computed(() =>
    Math.max(1, Math.ceil(filteredContracts.value.length / pageSize))
  );

  const pagedContracts = computed(() => {
    const s = (curPage.value - 1) * pageSize;
    return filteredContracts.value.slice(s, s + pageSize);
  });

  function onSearch() { curPage.value = 1; }

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
    searchText,
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
    loadData,
    toggleExpand,
  };
}
