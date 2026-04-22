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
  { key: 'sales', label: '销售表', icon: '📋' },
  { key: 'contract', label: '合同', icon: '📄' },
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
        :class="['nav-item', { active: activeMenu === menu.key }]"
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
  background: #16213e;
  border-right: 1px solid #0f3460;
  display: flex;
  flex-direction: column;
  height: 100vh;
}

.sidebar-logo {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 0 16px;
  height: 56px;
  border-bottom: 1px solid #0f3460;
  flex-shrink: 0;
}

.logo-icon { font-size: 22px; }

.logo-text {
  font-size: 15px;
  font-weight: 600;
  color: #aaa;
  letter-spacing: 0.5px;
}

.sidebar-nav {
  padding: 12px 8px;
  flex: 1;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  border-radius: 6px;
  cursor: pointer;
  color: #aaa;
  font-size: 14px;
  transition: all 0.2s;
  margin-bottom: 4px;
}

.nav-item:hover {
  background: #0f3460;
  color: #fff;
}

.nav-item.active {
  background: #4a9eff22;
  color: #4a9eff;
  border-left: 3px solid #4a9eff;
  padding-left: 9px;
}

.nav-icon {
  font-size: 16px;
  width: 20px;
  text-align: center;
}

.nav-label { font-weight: 500; }
</style>
