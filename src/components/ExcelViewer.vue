<script setup lang="ts">
import { computed, ref } from 'vue';

interface SheetInfo {
  name: string;
  index: number;
  row_count: number;
  col_count: number;
}

interface Props {
  data: string[][];
  sheets: SheetInfo[];
  currentSheetIndex: number;
  fileName: string;
}

const props = defineProps<Props>();
const emit = defineEmits<{
  (e: 'select-sheet', index: number): void;
  (e: 'import'): void;
}>();

const currentPage = ref(1);
const pageSize = ref(100);
const filterText = ref('');

const headers = computed(() => props.data[0] || []);
const rows = computed(() => props.data.slice(1) || []);
const totalPages = computed(() => Math.ceil(rows.value.length / pageSize.value));
const filteredRows = computed(() => {
  if (!filterText.value) return rows.value;
  const lower = filterText.value.toLowerCase();
  return rows.value.filter(row =>
    row.some(cell => cell.toLowerCase().includes(lower))
  );
});
const paginatedRows = computed(() => {
  const start = (currentPage.value - 1) * pageSize.value;
  return filteredRows.value.slice(start, start + pageSize.value);
});

function prevPage() {
  if (currentPage.value > 1) currentPage.value--;
}

function nextPage() {
  if (currentPage.value < totalPages.value) currentPage.value++;
}

function exportCsv() {
  const csv = [
    headers.value.join(','),
    ...paginatedRows.value.map(row => row.map(cell => `"${cell}"`).join(','))
  ].join('\n');

  const blob = new Blob(['\ufeff' + csv], { type: 'text/csv;charset=utf-8' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `${props.fileName}_sheet${props.currentSheetIndex}.csv`;
  a.click();
  URL.revokeObjectURL(url);
}
</script>

<template>
  <div class="excel-viewer">
    <header class="viewer-header">
      <div class="file-info">
        <span class="file-name">{{ fileName }}</span>
        <span class="sheet-tabs">
          <button
            v-for="(sheet, idx) in sheets"
            :key="idx"
            :class="['sheet-tab', { active: idx === currentSheetIndex }]"
            @click="emit('select-sheet', idx)"
          >
            {{ sheet.name }}
            <span class="sheet-info">({{ sheet.row_count }}行 x {{ sheet.col_count }}列)</span>
          </button>
        </span>
      </div>
      <div class="toolbar">
        <input
          v-model="filterText"
          type="text"
          placeholder="筛选..."
          class="filter-input"
        />
        <button class="tool-btn" @click="emit('import')">导入到库</button>
        <button class="tool-btn" @click="exportCsv">导出CSV</button>
      </div>
    </header>

    <div class="table-container">
      <table class="excel-table">
        <thead>
          <tr>
            <th class="row-num">#</th>
            <th v-for="(header, idx) in headers" :key="idx">{{ header }}</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="(row, rowIdx) in paginatedRows" :key="rowIdx">
            <td class="row-num">{{ (currentPage - 1) * pageSize + rowIdx + 1 }}</td>
            <td v-for="(cell, cellIdx) in row" :key="cellIdx">{{ cell }}</td>
          </tr>
        </tbody>
      </table>
    </div>

    <footer class="viewer-footer">
      <span class="row-count">
        共 {{ filteredRows.length }} 行
        <span v-if="filterText"> (筛选自 {{ rows.length }} 行)</span>
      </span>
      <div class="pagination">
        <button :disabled="currentPage === 1" @click="prevPage">上一页</button>
        <span>{{ currentPage }} / {{ totalPages || 1 }}</span>
        <button :disabled="currentPage >= totalPages" @click="nextPage">下一页</button>
      </div>
    </footer>
  </div>
</template>

<style scoped>
.excel-viewer {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #1a1a2e;
  border-radius: 8px;
  overflow: hidden;
}

.viewer-header {
  padding: 16px;
  background: #16213e;
  border-bottom: 1px solid #0f3460;
}

.file-info {
  margin-bottom: 12px;
}

.file-name {
  font-size: 18px;
  font-weight: bold;
  color: #fff;
  margin-bottom: 8px;
  display: block;
}

.sheet-tabs {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.sheet-tab {
  padding: 6px 12px;
  background: #0f3460;
  border: none;
  border-radius: 4px;
  color: #ccc;
  cursor: pointer;
  font-size: 13px;
  transition: all 0.2s;
}

.sheet-tab:hover {
  background: #1a4a7a;
  color: #fff;
}

.sheet-tab.active {
  background: #4a9eff;
  color: #fff;
}

.sheet-info {
  font-size: 11px;
  opacity: 0.7;
}

.toolbar {
  display: flex;
  gap: 12px;
  align-items: center;
}

.filter-input {
  padding: 8px 12px;
  background: #0f3460;
  border: 1px solid #1a4a7a;
  border-radius: 4px;
  color: #fff;
  font-size: 13px;
  width: 200px;
}

.filter-input:focus {
  outline: none;
  border-color: #4a9eff;
}

.tool-btn {
  padding: 8px 16px;
  background: #4a9eff;
  border: none;
  border-radius: 4px;
  color: white;
  cursor: pointer;
  font-size: 13px;
  transition: background 0.2s;
}

.tool-btn:hover {
  background: #3a8eef;
}

.table-container {
  flex: 1;
  overflow: auto;
}

.excel-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
}

.excel-table th,
.excel-table td {
  padding: 8px 12px;
  text-align: left;
  border: 1px solid #0f3460;
  white-space: nowrap;
}

.excel-table th {
  background: #16213e;
  color: #4a9eff;
  font-weight: 600;
  position: sticky;
  top: 0;
  z-index: 10;
}

.excel-table td {
  background: #1a1a2e;
  color: #ccc;
}

.excel-table tr:hover td {
  background: #16213e;
}

.row-num {
  width: 50px;
  background: #0f3460 !important;
  color: #666 !important;
  text-align: center;
  position: sticky;
  left: 0;
}

.viewer-footer {
  padding: 12px 16px;
  background: #16213e;
  border-top: 1px solid #0f3460;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.row-count {
  color: #888;
  font-size: 13px;
}

.pagination {
  display: flex;
  gap: 12px;
  align-items: center;
}

.pagination button {
  padding: 6px 12px;
  background: #0f3460;
  border: none;
  border-radius: 4px;
  color: #ccc;
  cursor: pointer;
  font-size: 13px;
}

.pagination button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.pagination button:hover:not(:disabled) {
  background: #1a4a7a;
  color: #fff;
}

.pagination span {
  color: #ccc;
  font-size: 13px;
}
</style>
