<script setup lang="ts">
import { ref } from 'vue';
import { open } from '@tauri-apps/api/dialog';
import { invoke } from '@tauri-apps/api/tauri';
import type { ExcelData, TabData } from './types';
import SheetViewer from './components/SheetViewer.vue';

const tabs     = ref<TabData[]>([]);
const activeId = ref<string>('');

// 选择 Sheet 弹窗
const sheetModal = ref<{ excel: ExcelData; resolve: (idx: number) => void } | null>(null);

function genId() {
  return 'tab_' + Math.random().toString(36).slice(2, 10);
}

async function openFile() {
  const path = await open({ filters: [{ name: 'Excel', extensions: ['xlsx', 'xls', 'xlsm'] }] }) as string | null;
  if (!path) return;
  try {
    const excel = await invoke<ExcelData>('open_excel', { path });
    if (excel.sheets.length === 1) {
      await createTab(excel, 0);
    } else {
      // 多 Sheet 弹窗选择
      const idx = await new Promise<number>(resolve => {
        sheetModal.value = { excel, resolve };
      });
      sheetModal.value = null;
      if (idx < 0) return;
      await createTab(excel, idx);
    }
  } catch (e) {
    alert(`打开失败: ${e}`);
  }
}

async function createTab(excel: ExcelData, sheetIndex: number) {
  const sheet = excel.sheets[sheetIndex];
  const id    = genId();
  const result = await invoke<{ rows: string[][], formulas: [number, string][] }>('read_sheet_data', { path: excel.file_path, sheetIndex });
  const raw = result.rows;
  const formulas = result.formulas;
  if (!raw || raw.length === 0) { alert('Sheet 无数据'); return; }
  const headers = raw[0].map((h, i) => h?.trim() || `列${i + 1}`);
  const rows    = raw.slice(1);
  await invoke('import_sheet', { tableName: id, headers, rows, formulas });
  const tab: TabData = {
    id,
    fileName:  excel.file_name,
    filePath:  excel.file_path,
    sheetName: sheet.name,
    sheetIndex,
    tableName: id,
    headers,
  };
  tabs.value.push(tab);
  activeId.value = id;
}

async function closeTab(id: string) {
  await invoke('drop_table', { tableName: id }).catch(() => {});
  const idx = tabs.value.findIndex(t => t.id === id);
  tabs.value.splice(idx, 1);
  if (activeId.value === id) {
    activeId.value = tabs.value[Math.max(0, idx - 1)]?.id ?? '';
  }
}
</script>

<template>
  <div class="app">
    <!-- 标签栏 -->
    <div class="tab-bar">
      <button class="btn-open" @click="openFile">📂 打开 Excel</button>
      <div class="tabs">
        <div
          v-for="tab in tabs"
          :key="tab.id"
          class="tab"
          :class="{ active: tab.id === activeId }"
          @click="activeId = tab.id"
        >
          <span class="tab-name">{{ tab.fileName }}</span>
          <span class="tab-sheet">{{ tab.sheetName }}</span>
          <button class="tab-close" @click.stop="closeTab(tab.id)">✕</button>
        </div>
      </div>
      <div v-if="tabs.length === 0" class="tab-hint">点击「打开 Excel」导入文件</div>
    </div>

    <!-- 内容区 -->
    <div class="content">
      <div v-if="tabs.length === 0" class="empty-state">
        <div class="empty-icon">📊</div>
        <p>打开 Excel 文件开始使用</p>
        <button class="btn-open-lg" @click="openFile">选择文件</button>
      </div>
      <template v-for="tab in tabs" :key="tab.id">
        <SheetViewer
          v-show="tab.id === activeId"
          :tab="tab"
        />
      </template>
    </div>

    <!-- Sheet 选择弹窗 -->
    <div v-if="sheetModal" class="modal-mask" @click.self="() => { sheetModal!.resolve(-1); sheetModal = null; }">
      <div class="sheet-modal">
        <div class="sheet-modal-header">
          <span>选择 Sheet</span>
          <button class="modal-close" @click="() => { sheetModal!.resolve(-1); sheetModal = null; }">✕</button>
        </div>
        <div class="sheet-list">
          <div
            v-for="sheet in sheetModal.excel.sheets"
            :key="sheet.index"
            class="sheet-item"
            @click="sheetModal!.resolve(sheet.index)"
          >
            <span class="sheet-item-name">{{ sheet.name }}</span>
            <span class="sheet-item-info">{{ sheet.row_count }} 行 × {{ sheet.col_count }} 列</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style>
* { margin: 0; padding: 0; box-sizing: border-box; }
body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Microsoft YaHei', sans-serif; background: #f0f2f5; height: 100vh; overflow: hidden; }
</style>

<style scoped>
.app { display: flex; flex-direction: column; height: 100vh; overflow: hidden; }

/* 标签栏 */
.tab-bar { display: flex; align-items: center; background: #fff; border-bottom: 1px solid #e8e8e8; height: 42px; padding: 0 8px; gap: 8px; flex-shrink: 0; overflow: hidden; }
.btn-open { padding: 4px 12px; background: #1677ff; border: none; border-radius: 4px; color: #fff; font-size: 13px; cursor: pointer; white-space: nowrap; flex-shrink: 0; }
.btn-open:hover { background: #4096ff; }
.tabs { display: flex; gap: 2px; overflow-x: auto; flex: 1; }
.tabs::-webkit-scrollbar { height: 3px; }
.tabs::-webkit-scrollbar-thumb { background: #d9d9d9; border-radius: 2px; }
.tab { display: flex; align-items: center; gap: 6px; padding: 0 10px; height: 34px; background: #f5f5f5; border: 1px solid #e8e8e8; border-radius: 4px 4px 0 0; cursor: pointer; white-space: nowrap; font-size: 13px; color: #595959; flex-shrink: 0; }
.tab:hover { background: #e6f4ff; }
.tab.active { background: #fff; border-bottom-color: #fff; color: #1677ff; font-weight: 500; }
.tab-name { max-width: 120px; overflow: hidden; text-overflow: ellipsis; }
.tab-sheet { font-size: 11px; color: #8c8c8c; background: #f0f0f0; padding: 1px 5px; border-radius: 3px; }
.tab.active .tab-sheet { background: #e6f4ff; color: #1677ff; }
.tab-close { background: none; border: none; color: #bfbfbf; cursor: pointer; font-size: 12px; padding: 0 2px; line-height: 1; }
.tab-close:hover { color: #ff4d4f; }
.tab-hint { font-size: 12px; color: #bfbfbf; }

/* 内容区 */
.content { flex: 1; overflow: hidden; }
.empty-state { display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100%; gap: 16px; color: #bfbfbf; }
.empty-icon { font-size: 56px; }
.empty-state p { font-size: 15px; }
.btn-open-lg { padding: 8px 28px; background: #1677ff; border: none; border-radius: 6px; color: #fff; font-size: 14px; cursor: pointer; }
.btn-open-lg:hover { background: #4096ff; }

/* Sheet 选择弹窗 */
.modal-mask { position: fixed; inset: 0; background: rgba(0,0,0,0.45); z-index: 1000; display: flex; align-items: center; justify-content: center; }
.sheet-modal { background: #fff; border-radius: 8px; width: 360px; max-height: 70vh; display: flex; flex-direction: column; box-shadow: 0 8px 32px rgba(0,0,0,0.18); }
.sheet-modal-header { display: flex; align-items: center; justify-content: space-between; padding: 14px 18px; border-bottom: 1px solid #f0f0f0; font-size: 15px; font-weight: 600; color: #262626; }
.modal-close { background: none; border: none; font-size: 16px; color: #8c8c8c; cursor: pointer; }
.modal-close:hover { color: #ff4d4f; }
.sheet-list { overflow-y: auto; padding: 8px; }
.sheet-item { display: flex; align-items: center; justify-content: space-between; padding: 10px 12px; border-radius: 6px; cursor: pointer; }
.sheet-item:hover { background: #e6f4ff; }
.sheet-item-name { font-size: 14px; color: #262626; }
.sheet-item-info { font-size: 12px; color: #8c8c8c; }
</style>
