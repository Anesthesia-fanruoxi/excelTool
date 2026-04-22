<script setup lang="ts">
interface MenuItem {
  key: string;
  label: string;
  icon: string;
}

interface Props {
  activeMenu: string;
}

defineProps<Props>();
const emit = defineEmits<{
  (e: 'navigate', key: string): void;
}>();

const menus: MenuItem[] = [
  { key: 'data', label: '数据管理', icon: '🗄️' },
  { key: 'sales', label: '销售明细', icon: '📋' },
  { key: 'contract', label: '合同管理', icon: '📄' },
];
</script>

<template>
  <aside class="sidebar">
    <div class="sidebar-logo">
      <span class="logo-icon">⚡</span>
      <span class="logo-text">Excel Tool</span>
    </div>
    <nav class="sidebar-nav">
      <div
        v-for="menu in menus"
        :key="menu.key"
        :class="['nav-item', { 'nav-active': activeMenu === menu.key }]"
        @click="emit('navigate', menu.key)"
      >
        <span class="nav-icon">{{ menu.icon }}</span>
        <span class="nav-label">{{ menu.label }}</span>
      </div>
    </nav>
  </aside>
</template>

<style scoped>
.sidebar {
  width: 200px;
  min-width: 200px;
  background: #fff;
  border-right: 1px solid #e8e8e8;
  display: flex;
  flex-direction: column;
  height: 100vh;
  box-shadow: 2px 0 8px rgba(0,0,0,0.06);
}

.sidebar-logo {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 0 16px;
  height: 56px;
  border-bottom: 1px solid #f0f0f0;
  flex-shrink: 0;
}
.logo-icon { font-size: 20px; }
.logo-text {
  font-size: 15px;
  font-weight: 600;
  color: #1677ff;
  letter-spacing: 0.5px;
}

.sidebar-nav { padding: 8px 8px; flex: 1; }

.nav-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 9px 12px;
  border-radius: 6px;
  cursor: pointer;
  color: #595959;
  font-size: 14px;
  transition: all 0.15s;
  margin-bottom: 2px;
}
.nav-item:hover { background: #f0f5ff; color: #1677ff; }
.nav-active {
  background: #e6f4ff;
  color: #1677ff;
  font-weight: 600;
  border-left: 3px solid #1677ff;
  padding-left: 9px;
}
.nav-icon { font-size: 15px; width: 20px; text-align: center; }
.nav-label { font-weight: 500; }
</style>
