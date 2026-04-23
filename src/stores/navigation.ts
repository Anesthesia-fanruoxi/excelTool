import { defineStore } from 'pinia';
import { ref } from 'vue';

export const useNavigationStore = defineStore('navigation', () => {
  const activeMenu     = ref('data');
  const jumpContractNo = ref('');

  function setMenu(key: string) {
    activeMenu.value = key;
  }

  function navigateToContract(contractNo: string) {
    jumpContractNo.value = contractNo;
    activeMenu.value     = 'contract';
  }

  function clearJump() {
    jumpContractNo.value = '';
  }

  return { activeMenu, jumpContractNo, setMenu, navigateToContract, clearJump };
});
