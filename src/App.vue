<script setup lang="ts">
import SideBar from './components/SideBar.vue';
import DataManager from './views/DataManager.vue';
import SalesView from './views/SalesView.vue';
import ContractView from './views/ContractView.vue';
import QuoteView from './views/QuoteView.vue';
import { useNavigationStore } from './stores/navigation';
import { storeToRefs } from 'pinia';

const nav = useNavigationStore();
const { activeMenu } = storeToRefs(nav);
</script>

<template>
  <div class="app-container">
    <SideBar :active-menu="activeMenu" @navigate="nav.setMenu($event)" />
    <main class="main-content">
      <DataManager v-if="activeMenu === 'data'" />
      <SalesView v-else-if="activeMenu === 'sales'" />
      <ContractView v-else-if="activeMenu === 'contract'" />
      <QuoteView v-else-if="activeMenu === 'quote'" />
      <div v-else-if="activeMenu === 'sign'" class="coming-soon">签收对账功能开发中...</div>
    </main>
  </div>
</template>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Microsoft YaHei', sans-serif;
  background: #f0f2f5;
  color: #333;
  height: 100vh;
  overflow: hidden;
}

.app-container {
  display: flex;
  height: 100vh;
  overflow: hidden;
}

.main-content {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  background: #f0f2f5;
}
.coming-soon {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  font-size: 15px;
  color: #bfbfbf;
}
</style>
