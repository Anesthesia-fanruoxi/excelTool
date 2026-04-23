import { ref, computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import { type ContractRow, type ContractDetailRow } from '../constants/columns';

export function useContractView() {
  const isLoading = ref(false);
  const contracts = ref<ContractRow[]>([]);
  const keyword   = ref('');
  const applied   = ref('');
  const curPage   = ref(1);
  const pageSize  = 50;

  const expandedNo   = ref<string | null>(null);
  const expandedRows = ref<Record<string, ContractDetailRow[]>>({});

  const toastMsg  = ref('');
  const toastType = ref<'success' | 'error'>('success');
  let toastTimer: ReturnType<typeof setTimeout> | null = null;

  async function loadData() {
    isLoading.value = true;
    try {
      contracts.value    = await invoke<ContractRow[]>('query_contracts', { keyword: applied.value });
      expandedRows.value = {};
      expandedNo.value   = null;
    } catch (e) {
      showToast(`加载失败: ${e}`, 'error');
    } finally {
      isLoading.value = false;
    }
  }

  const totalPages = computed(() => Math.max(1, Math.ceil(contracts.value.length / pageSize)));

  const pagedContracts = computed(() => {
    const s = (curPage.value - 1) * pageSize;
    return contracts.value.slice(s, s + pageSize);
  });

  function onSearch() { applied.value = keyword.value; curPage.value = 1; loadData(); }
  function onReset()  { keyword.value = ''; applied.value = ''; curPage.value = 1; loadData(); }

  async function toggleExpand(contractNo: string) {
    if (expandedNo.value === contractNo) { expandedNo.value = null; return; }
    expandedNo.value = contractNo;
    if (!expandedRows.value[contractNo]) {
      try {
        expandedRows.value[contractNo] = await invoke<ContractDetailRow[]>(
          'query_contract_detail', { contractNo }
        );
      } catch (e) {
        showToast(`加载明细失败: ${e}`, 'error');
      }
    }
  }

  function showToast(msg: string, type: 'success' | 'error') {
    toastMsg.value  = msg;
    toastType.value = type;
    if (toastTimer) clearTimeout(toastTimer);
    toastTimer = setTimeout(() => (toastMsg.value = ''), type === 'error' ? 5000 : 2500);
  }

  onMounted(loadData);

  return {
    isLoading, contracts, keyword, curPage, pageSize,
    totalPages, pagedContracts, expandedNo, expandedRows,
    toastMsg, toastType,
    onSearch, onReset, loadData, toggleExpand,
  };
}
