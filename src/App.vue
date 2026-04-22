<script setup lang="ts">
import { ref } from 'vue';
import SideBar from './components/SideBar.vue';
import DataManager from './views/DataManager.vue';
import SalesView from './views/SalesView.vue';
import ContractView from './views/ContractView.vue';

const activeMenu = ref('data');
</script>

<template>
  <div class="app-container">
    <SideBar :active-menu="activeMenu" @navigate="activeMenu = $event" />
    <main class="main-content">
      <DataManager v-if="activeMenu === 'data'" />
      <SalesView v-else-if="activeMenu === 'sales'" />
      <ContractView v-else-if="activeMenu === 'contract'" />
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
</style>
