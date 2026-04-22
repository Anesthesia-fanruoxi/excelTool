import { defineStore } from 'pinia';
import { ref } from 'vue';

export const useNavigationStore = defineStore('navigation', () => {
  const activeMenu = ref('data');
  // 跳转到合同管理时携带的合同号，消费后清空
  const jumpContractNo = ref('');

  function navigateToContract(contractNo: string) {
    jumpContractNo.value = contractNo;
    activeMenu.value = 'contract';
  }

  function clearJump() {
    jumpContractNo.value = '';
  }

  return { activeMenu, jumpContractNo, navigateToContract, clearJump };
});
