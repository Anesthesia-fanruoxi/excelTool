<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted, computed } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import { save } from '@tauri-apps/api/dialog';
import type { TabData, PageResult } from '../types';

const props = defineProps<{ tab: TabData }>();

const PAGE_SIZE = 50;

const columns    = ref<string[]>([]);
const rows       = ref<(string | null)[][]>([]);
const rowIds     = ref<number[]>([]);
const total      = ref(0);
const curPage    = ref(1);
const isLoading  = ref(false);
const totalPages = ref(1);

// 关键字搜索条件
interface Filter { col: string; keyword: string }
const filters = ref<Filter[]>([]);
const appliedFilters = ref<Filter[]>([]);

// 列筛选条件 (col -> Set<val>)
const colFilterMap = ref<Record<string, Set<string>>>({});
const appliedColFilterMap = ref<Record<string, string[]>>({});

// 下拉面板状态
const dropdown = ref<{
  col: string;
  colIdx: number;
  vals: string[];
  counts: Record<string, number>;
  checked: Set<string>;
  search: string;
  x: number;
  y: number;
} | null>(null);

// 隐藏列
const hiddenCols = ref<Set<string>>(new Set());
const showColPanel = ref(false);

function toggleColPanel(e: MouseEvent) {
  e.stopPropagation();
  showColPanel.value = !showColPanel.value;
}

function toggleHideCol(col: string) {
  if (hiddenCols.value.has(col)) hiddenCols.value.delete(col);
  else hiddenCols.value.add(col);
  hiddenCols.value = new Set(hiddenCols.value); // 触发响应式
}

function showAllCols() {
  hiddenCols.value = new Set();
}

function hideAllCols() {
  hiddenCols.value = new Set(columns.value);
}

const visibleColumns = computed(() => columns.value.filter(c => !hiddenCols.value.has(c)));
const visibleColIndices = computed(() => {
  const indices: number[] = [];
  columns.value.forEach((c, i) => { if (!hiddenCols.value.has(c)) indices.push(i); });
  return indices;
});

function displayCol(col: string) {
  return col.endsWith('[公式]') ? col.slice(0, -4) : col;
}
async function openDropdown(e: MouseEvent, col: string, colIdx: number) {
  e.stopPropagation();
  if (dropdown.value?.col === col) { dropdown.value = null; return; }
  const rect = (e.currentTarget as HTMLElement).closest('.th')!.getBoundingClientRect();
  const raw = await invoke<[string, number][]>('get_distinct_values', {
    tableName: props.tab.tableName,
    column: col,
  });
  const vals = raw.map(([v]) => v);
  const counts: Record<string, number> = {};
  raw.forEach(([v, c]) => { counts[v] = c; });
  const existing = appliedColFilterMap.value[col];
  const panelW = 320;
  let x = rect.left;
  if (x + panelW > window.innerWidth - 8) x = window.innerWidth - panelW - 8;
  dropdown.value = {
    col, colIdx, vals, counts,
    checked: existing ? new Set(existing) : new Set(vals),
    search: '',
    x,
    y: rect.bottom + 2,
  };
  sortBy.value = 'default';
}

function closeDropdown() {
  if (editing.value) return;
  dropdown.value = null;
}

const filteredDropdownVals = computed(() => {
  if (!dropdown.value) return [];
  const s = dropdown.value.search.toLowerCase();
  let list = dropdown.value.vals.filter(v => v !== '__EMPTY__');
  if (s) list = list.filter(v => v.toLowerCase().includes(s));
  if (sortBy.value === 'val-asc') list.sort((a, b) => a.localeCompare(b, 'zh'));
  else if (sortBy.value === 'val-desc') list.sort((a, b) => b.localeCompare(a, 'zh'));
  else if (sortBy.value === 'count-asc') list.sort((a, b) => (dropdown.value!.counts[a] ?? 0) - (dropdown.value!.counts[b] ?? 0));
  else if (sortBy.value === 'count-desc') list.sort((a, b) => (dropdown.value!.counts[b] ?? 0) - (dropdown.value!.counts[a] ?? 0));
  // 空值始终排在末尾
  if (!s && dropdown.value.vals.includes('__EMPTY__')) list.push('__EMPTY__');
  return list;
});

const sortBy = ref<'default' | 'val-asc' | 'val-desc' | 'count-asc' | 'count-desc'>('default');

function toggleSort(type: 'val' | 'count') {
  if (type === 'val') {
    if (sortBy.value === 'val-asc') sortBy.value = 'val-desc';
    else sortBy.value = 'val-asc';
  } else {
    if (sortBy.value === 'count-desc') sortBy.value = 'count-asc';
    else sortBy.value = 'count-desc';
  }
}

function toggleVal(val: string) {
  if (!dropdown.value) return;
  if (dropdown.value.checked.has(val)) dropdown.value.checked.delete(val);
  else dropdown.value.checked.add(val);
}

function selectAll() {
  if (!dropdown.value) return;
  dropdown.value.vals.forEach(v => dropdown.value!.checked.add(v));
}

function clearAll() {
  if (!dropdown.value) return;
  dropdown.value.checked.clear();
}

function applyDropdown() {
  if (!dropdown.value) return;
  const { col, vals, checked } = dropdown.value;
  // 全选 = 不加条件；否则记录选中值
  if (checked.size === vals.length) {
    delete appliedColFilterMap.value[col];
  } else {
    appliedColFilterMap.value[col] = [...checked];
  }
  dropdown.value = null;
  curPage.value = 1;
  loadPage();
}

function hasColFilter(col: string) {
  return !!appliedColFilterMap.value[col];
}

// 关键字搜索
function addFilter() {
  filters.value.push({ col: columns.value[0] ?? '', keyword: '' });
}
function removeFilter(idx: number) { filters.value.splice(idx, 1); }

function onSearch() {
  appliedFilters.value = filters.value.filter(f => f.col && f.keyword.trim()).map(f => ({ ...f }));
  curPage.value = 1;
  loadPage();
}

function onReset() {
  filters.value = [];
  appliedFilters.value = [];
  appliedColFilterMap.value = {};
  curPage.value = 1;
  loadPage();
}

// 编辑
const editing = ref<{ rowIdx: number; colIdx: number } | null>(null);
const editVal = ref('');

// 列宽
const SEQ_WIDTH = 46;
const DEFAULT_COL_WIDTH = 120;
const MIN_COL_WIDTH = 40;
const colWidths = ref<number[]>([]);

function initColWidths() {
  if (colWidths.value.length !== columns.value.length) {
    colWidths.value = columns.value.map(() => DEFAULT_COL_WIDTH);
  }
}

// 拖拽列宽
let dragColIdx = -1;
let dragStartX = 0;
let dragStartW = 0;
const isDragging = ref(false);

function startResize(e: MouseEvent, idx: number) {
  e.preventDefault();
  dragColIdx = idx;
  dragStartX = e.clientX;
  dragStartW = colWidths.value[idx] ?? DEFAULT_COL_WIDTH;
  isDragging.value = true;
  window.addEventListener('mousemove', doResize);
  window.addEventListener('mouseup', stopResize);
}

function doResize(e: MouseEvent) {
  if (dragColIdx < 0) return;
  colWidths.value[dragColIdx] = Math.max(MIN_COL_WIDTH, dragStartW + (e.clientX - dragStartX));
}

function stopResize() {
  dragColIdx = -1;
  isDragging.value = false;
  window.removeEventListener('mousemove', doResize);
  window.removeEventListener('mouseup', stopResize);
}

// 双击自适应列宽
function autoFitCol(idx: number) {
  const col = columns.value[idx];
  const headerText = displayCol(col) + (col.endsWith('[公式]') ? '  fx' : '');
  // 用 canvas 测量文字宽度
  const canvas = document.createElement('canvas');
  const ctx = canvas.getContext('2d')!;
  ctx.font = '600 13px -apple-system, BlinkMacSystemFont, Segoe UI, Microsoft YaHei, sans-serif';
  let maxW = ctx.measureText(headerText).width + 40; // 表头加 padding + 筛选箭头
  ctx.font = '13px -apple-system, BlinkMacSystemFont, Segoe UI, Microsoft YaHei, sans-serif';
  rows.value.forEach(row => {
    const text = row[idx] ?? '';
    const w = ctx.measureText(text).width + 24;
    if (w > maxW) maxW = w;
  });
  colWidths.value[idx] = Math.min(Math.max(maxW, MIN_COL_WIDTH), 500);
}

onUnmounted(() => {
  stopResize();
  document.removeEventListener('click', closeDropdown);
});

// 公式列
function isFormulaCol(colIdx: number) {
  return columns.value[colIdx]?.endsWith('[公式]') ?? false;
}

async function loadPage() {
  if (!props.tab.tableName) return;
  isLoading.value = true;
  try {
    const filterPairs = appliedFilters.value.map(f => [f.col, f.keyword] as [string, string]);
    const colFilterPairs = Object.entries(appliedColFilterMap.value).map(([col, vals]) => [col, vals] as [string, string[]]);
    const [result, ids] = await Promise.all([
      invoke<PageResult>('query_page', {
        tableName: props.tab.tableName,
        page: curPage.value,
        pageSize: PAGE_SIZE,
        filters: filterPairs,
        colFilters: colFilterPairs,
      }),
      invoke<number[]>('get_row_ids', {
        tableName: props.tab.tableName,
        page: curPage.value,
        pageSize: PAGE_SIZE,
        filters: filterPairs,
        colFilters: colFilterPairs,
      }),
    ]);
    columns.value    = result.columns;
    rows.value       = result.rows;
    rowIds.value     = ids;
    total.value      = result.total;
    totalPages.value = Math.max(1, Math.ceil(result.total / PAGE_SIZE));
    initColWidths();
  } catch (e) {
    console.error(e);
  } finally {
    isLoading.value = false;
  }
}

function startEdit(rowIdx: number, colIdx: number) {
  console.log('[edit] dblclick', rowIdx, colIdx, 'isFormula:', isFormulaCol(colIdx));
  if (isFormulaCol(colIdx)) return;
  closeDropdown();
  editing.value = { rowIdx, colIdx };
  editVal.value = rows.value[rowIdx][colIdx] ?? '';
}

async function commitEdit() {
  if (!editing.value) return;
  const { rowIdx, colIdx } = editing.value;
  const rowId = rowIds.value[rowIdx];
  const col   = columns.value[colIdx];
  const val   = editVal.value;
  editing.value = null;
  try {
    const recalcResults = await invoke<[number, string][]>('update_cell', {
      tableName: props.tab.tableName,
      rowId, column: col, value: val,
    });
    // 更新普通列
    rows.value[rowIdx][colIdx] = val;
    // 更新重算后的公式列
    for (const [ci, newVal] of recalcResults) {
      rows.value[rowIdx][ci] = newVal || null;
    }
  } catch (e) {
    alert(`保存失败: ${e}`);
  }
}

function cancelEdit() { editing.value = null; }

async function exportExcel() {
  const savePath = await save({
    filters: [{ name: 'Excel', extensions: ['xlsx'] }],
    defaultPath: props.tab.fileName.replace(/\.[^.]+$/, '') + '_导出.xlsx',
  });
  if (!savePath) return;
  try {
    await invoke('export_excel', { tableName: props.tab.tableName, savePath });
    alert('导出成功');
  } catch (e) {
    alert(`导出失败: ${e}`);
  }
}

watch(() => props.tab.tableName, () => {
  filters.value = [];
  appliedFilters.value = {};
  appliedColFilterMap.value = {};
  colWidths.value = [];
  hiddenCols.value = new Set();
  loadPage();
});

onMounted(() => {
  loadPage();
  document.addEventListener('click', () => {
    closeDropdown();
    showColPanel.value = false;
  });
});
</script>

<template>
  <div class="viewer" :style="isDragging ? 'cursor:col-resize;user-select:none' : ''">

    <!-- 工具栏 -->
    <div class="toolbar-wrap">
      <!-- 第一行：操作按钮 -->
      <div class="toolbar-main">
        <button class="btn-add-filter" @click="addFilter">＋ 新增搜索条件</button>
        <!-- 列设置按钮 -->
        <div class="col-panel-wrap" @click.stop>
          <button class="btn-col-setting" @click="toggleColPanel">
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><rect x="1" y="2" width="12" height="1.5" rx="0.75" fill="currentColor"/><rect x="1" y="6.25" width="12" height="1.5" rx="0.75" fill="currentColor"/><rect x="1" y="10.5" width="12" height="1.5" rx="0.75" fill="currentColor"/><rect x="3.5" y="0.5" width="1.5" height="4" rx="0.75" fill="currentColor"/><rect x="9" y="4.75" width="1.5" height="4" rx="0.75" fill="currentColor"/></svg>
            列设置
            <span v-if="hiddenCols.size > 0" class="col-hidden-badge">{{ hiddenCols.size }}</span>
          </button>
          <!-- 列设置面板 -->
          <div v-if="showColPanel" class="col-panel">
            <div class="col-panel-header">
              <span>显示/隐藏列</span>
              <div style="display:flex;gap:8px">
                <span class="cd-link" @click="showAllCols">全显</span>
                <span class="cd-link" @click="hideAllCols">全隐</span>
              </div>
            </div>
            <div class="col-panel-list">
              <label v-for="col in columns" :key="col" class="col-panel-item">
                <input type="checkbox" :checked="!hiddenCols.has(col)" @change="toggleHideCol(col)" />
                <span>{{ displayCol(col) }}</span>
                <span v-if="col.endsWith('[公式]')" class="formula-badge" style="margin-left:4px">fx</span>
              </label>
            </div>
          </div>
        </div>
        <button v-if="Object.keys(appliedColFilterMap).length > 0 && filters.length === 0" class="btn-reset" @click="onReset">清除筛选</button>
        <span class="stat">共 <b>{{ total }}</b> 行</span>
        <button class="btn-export" @click="exportExcel">导出 Excel</button>
      </div>

      <!-- 第二行：搜索条件（有条件时才显示） -->
      <div v-if="filters.length > 0" class="toolbar-search">
        <div class="filter-list">
          <div v-for="(f, idx) in filters" :key="idx" class="filter-item">
            <select v-model="f.col" class="col-select">
              <option v-for="c in columns" :key="c" :value="c">{{ displayCol(c) }}</option>
            </select>
            <input v-model="f.keyword" class="search-inp" placeholder="关键字..." @keydown.enter="onSearch" />
            <button class="btn-remove" @click="removeFilter(idx)">✕</button>
          </div>
        </div>
        <div class="search-actions">
          <button class="btn-search" @click="onSearch">搜索</button>
          <button class="btn-reset" @click="onReset">重置</button>
        </div>
      </div>
    </div>

    <!-- 表格 -->
    <div class="table-wrap" @click="closeDropdown">
      <div v-if="isLoading" class="loading">加载中...</div>
      <div v-else-if="rows.length === 0" class="empty">暂无数据</div>
      <div v-else class="scroll-wrap">
        <div class="scroll-inner">
          <!-- 表头 -->
          <div class="thead-row">
            <div class="th col-seq">#</div>
            <div
              v-for="col in visibleColumns"
              :key="col"
              class="th"
              :class="{ 'th-formula': col.endsWith('[公式]'), 'th-filtered': hasColFilter(col) }"
              :style="{ width: colWidths[columns.indexOf(col)] + 'px', minWidth: colWidths[columns.indexOf(col)] + 'px' }"
            >
              <span class="th-text">{{ displayCol(col) }}</span>
              <span v-if="col.endsWith('[公式]')" class="formula-badge">fx</span>
              <span
                class="filter-arrow"
                :class="{ 'filter-arrow-active': hasColFilter(col) }"
                @click.stop="openDropdown($event, col, columns.indexOf(col))"
              >
                <svg width="12" height="12" viewBox="0 0 12 12" fill="none" xmlns="http://www.w3.org/2000/svg">
                  <rect x="1" y="2" width="10" height="1.5" rx="0.75" fill="currentColor"/>
                  <rect x="2.5" y="5" width="7" height="1.5" rx="0.75" fill="currentColor"/>
                  <rect x="4.5" y="8" width="3" height="1.5" rx="0.75" fill="currentColor"/>
                </svg>
              </span>
              <span class="resize-handle" @mousedown.stop="startResize($event, columns.indexOf(col))" @dblclick.stop="autoFitCol(columns.indexOf(col))" />
            </div>
          </div>

          <!-- 数据行 -->
          <div v-for="(row, rIdx) in rows" :key="rowIds[rIdx]" class="tbody-row">
            <div class="td col-seq">{{ (curPage - 1) * PAGE_SIZE + rIdx + 1 }}</div>
            <div
              v-for="cIdx in visibleColIndices"
              :key="cIdx"
              class="td cell"
              :class="{ 'cell-formula': isFormulaCol(cIdx) }"
              :style="{ width: colWidths[cIdx] + 'px', minWidth: colWidths[cIdx] + 'px' }"
              @dblclick.stop="startEdit(rIdx, cIdx)"
            >
              <template v-if="editing && editing.rowIdx === rIdx && editing.colIdx === cIdx">
                <input v-model="editVal" class="cell-inp" autofocus @blur="commitEdit" @keydown.enter="commitEdit" @keydown.esc="cancelEdit" />
              </template>
              <template v-else>{{ row[cIdx] ?? '' }}</template>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 列筛选下拉面板 -->
    <Teleport to="body">
      <div v-if="dropdown" class="col-dropdown" :style="{ left: dropdown.x + 'px', top: dropdown.y + 'px' }" @click.stop>
        <div class="cd-search">
          <input v-model="dropdown.search" placeholder="搜索..." class="cd-search-inp" autofocus />
        </div>
        <div class="cd-actions">
          <span class="cd-link" @click="selectAll">全选</span>
          <span class="cd-link" @click="clearAll">清空</span>
          <span class="cd-sort" :class="{ active: sortBy.startsWith('val') }" @click="toggleSort('val')">
            按值{{ sortBy === 'val-asc' ? ' ↑' : sortBy === 'val-desc' ? ' ↓' : '' }}
          </span>
          <span class="cd-sort" :class="{ active: sortBy.startsWith('count') }" @click="toggleSort('count')">
            按数量{{ sortBy === 'count-asc' ? ' ↑' : sortBy === 'count-desc' ? ' ↓' : '' }}
          </span>
        </div>
        <div class="cd-list">
          <label v-for="val in filteredDropdownVals" :key="val" class="cd-item">
            <input type="checkbox" :checked="dropdown.checked.has(val)" @change="toggleVal(val)" />
            <span class="cd-item-text" :class="{ 'cd-empty-val': val === '__EMPTY__' }">{{ val === '__EMPTY__' ? '(空)' : val }}</span>
            <span class="cd-item-count">({{ dropdown.counts[val] }})</span>
          </label>
          <div v-if="filteredDropdownVals.length === 0" class="cd-empty">无匹配项</div>
        </div>
        <div class="cd-footer">
          <button class="cd-btn-ok" @click="applyDropdown">确定</button>
          <button class="cd-btn-cancel" @click="closeDropdown">取消</button>
        </div>
      </div>
    </Teleport>

    <!-- 分页 -->
    <div class="pagination">
      <button :disabled="curPage <= 1" @click="curPage--; loadPage()">上一页</button>
      <span>{{ curPage }} / {{ totalPages }}</span>
      <button :disabled="curPage >= totalPages" @click="curPage++; loadPage()">下一页</button>
    </div>
  </div>
</template>

<style scoped>
.viewer { display: flex; flex-direction: column; height: 100%; overflow: hidden; background: #f0f2f5; }

/* 工具栏 */
.toolbar-wrap { background: #fff; border-bottom: 1px solid #e8e8e8; flex-shrink: 0; }
.toolbar-main { display: flex; align-items: center; gap: 8px; padding: 7px 12px; }
.toolbar-search { display: flex; align-items: flex-start; gap: 8px; padding: 6px 12px 8px; border-top: 1px solid #f0f0f0; background: #fafafa; }
.search-actions { display: flex; gap: 6px; flex-shrink: 0; align-self: flex-start; }
.btn-add-filter { padding: 5px 12px; background: #fff; border: 1px dashed #1677ff; border-radius: 4px; color: #1677ff; font-size: 13px; cursor: pointer; white-space: nowrap; flex-shrink: 0; }
.btn-add-filter:hover { background: #e6f4ff; }
.col-panel-wrap { position: relative; flex-shrink: 0; }
.btn-col-setting { display: flex; align-items: center; gap: 5px; padding: 5px 12px; background: #fff; border: 1px solid #d9d9d9; border-radius: 4px; color: #595959; font-size: 13px; cursor: pointer; white-space: nowrap; }
.btn-col-setting:hover { border-color: #1677ff; color: #1677ff; }
.col-hidden-badge { background: #1677ff; color: #fff; border-radius: 10px; font-size: 11px; padding: 0 6px; margin-left: 2px; }
.col-panel { position: absolute; top: calc(100% + 4px); left: 0; z-index: 999; background: #fff; border: 1px solid #e0e0e0; border-radius: 6px; box-shadow: 0 4px 16px rgba(0,0,0,0.12); width: 200px; }
.col-panel-header { display: flex; align-items: center; justify-content: space-between; padding: 8px 12px; border-bottom: 1px solid #f0f0f0; font-size: 13px; font-weight: 600; color: #262626; }
.col-panel-list { max-height: 280px; overflow-y: auto; padding: 4px 0; }
.col-panel-item { display: flex; align-items: center; gap: 8px; padding: 6px 12px; cursor: pointer; font-size: 13px; color: #262626; }
.col-panel-item:hover { background: #e6f4ff; }
.col-panel-item input { accent-color: #1677ff; cursor: pointer; }
.filter-list { display: grid; grid-template-columns: repeat(3, 1fr); gap: 6px; flex: 1; }
.filter-item { display: flex; align-items: center; gap: 6px; }
.col-select { padding: 4px 6px; border: 1px solid #d9d9d9; border-radius: 4px; font-size: 13px; color: #333; width: 90px; }
.col-select:focus { outline: none; border-color: #1677ff; }
.search-inp { padding: 5px 10px; border: 1px solid #d9d9d9; border-radius: 4px; font-size: 13px; width: 100%; box-sizing: border-box; }
.search-inp:focus { outline: none; border-color: #1677ff; }
.btn-remove { background: none; border: none; color: #bfbfbf; cursor: pointer; font-size: 13px; padding: 0 4px; }
.btn-remove:hover { color: #ff4d4f; }
.btn-search { padding: 5px 12px; background: #1677ff; border: none; border-radius: 4px; color: #fff; font-size: 13px; cursor: pointer; align-self: flex-start; }
.btn-search:hover { background: #4096ff; }
.btn-reset { padding: 5px 12px; background: #fff; border: 1px solid #d9d9d9; border-radius: 4px; color: #595959; font-size: 13px; cursor: pointer; align-self: flex-start; }
.btn-reset:hover { border-color: #1677ff; color: #1677ff; }
.stat { font-size: 12px; color: #8c8c8c; align-self: center; }
.btn-export { padding: 5px 12px; background: #52c41a; border: none; border-radius: 4px; color: #fff; font-size: 13px; cursor: pointer; white-space: nowrap; }
.btn-export:hover { background: #73d13d; }
.stat b { color: #1677ff; }

.table-wrap { flex: 1; overflow: hidden; background: #fff; }
.loading, .empty { display: flex; align-items: center; justify-content: center; height: 100%; color: #bfbfbf; font-size: 14px; }
.scroll-wrap { height: 100%; overflow: auto; }
.scroll-inner { display: inline-flex; flex-direction: column; min-width: 100%; }

.thead-row { display: flex; position: sticky; top: 0; z-index: 10; border-bottom: 2px solid #e8e8e8; background: #fafafa; }
.tbody-row { display: flex; border-bottom: 1px solid #f0f0f0; }
.tbody-row:hover .td { background: #e6f4ff; }

.th, .td { flex-shrink: 0; padding: 4px 10px; font-size: 13px; border-right: 1px solid #f0f0f0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; box-sizing: border-box; }
.th:last-child, .td:last-child { border-right: none; }
.th { position: relative; display: flex; align-items: center; gap: 4px; background: #fafafa; color: #595959; font-weight: 600; user-select: none; height: 30px; padding-right: 22px; }
.th-text { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.th-formula { color: #389e0d; }
.th-filtered { background: #e6f4ff !important; }

.filter-arrow { position: absolute; right: 8px; top: 50%; transform: translateY(-50%); display: flex; align-items: center; justify-content: center; width: 18px; height: 18px; border-radius: 3px; color: #bfbfbf; cursor: pointer; transition: background 0.15s, color 0.15s; }
.filter-arrow:hover { background: #e6f4ff; color: #1677ff; }
.filter-arrow-active { color: #1677ff; background: #e6f4ff; }

.col-seq { width: 46px !important; min-width: 46px !important; text-align: center; color: #bfbfbf; background: #fafafa; font-size: 12px; justify-content: center; padding-right: 10px; }
.cell { cursor: default; background: #fff; }
.cell-formula { background: #f6ffed !important; cursor: not-allowed; }
.formula-badge { flex-shrink: 0; display: inline-block; padding: 0 3px; background: #389e0d; color: #fff; border-radius: 3px; font-size: 10px; font-family: monospace; }
.cell-inp { width: 100%; padding: 2px 4px; border: 2px solid #1677ff; border-radius: 2px; font-size: 13px; outline: none; background: #fff; }

.resize-handle { position: absolute; right: 0; top: 0; width: 6px; height: 100%; cursor: col-resize; z-index: 1; }
.resize-handle:hover { background: rgba(22, 119, 255, 0.4); }

.pagination { display: flex; align-items: center; justify-content: center; gap: 12px; padding: 8px; background: #fff; border-top: 1px solid #f0f0f0; flex-shrink: 0; }
.pagination button { padding: 4px 14px; background: #fff; border: 1px solid #d9d9d9; border-radius: 4px; color: #595959; font-size: 13px; cursor: pointer; }
.pagination button:disabled { opacity: 0.4; cursor: not-allowed; }
.pagination button:hover:not(:disabled) { border-color: #1677ff; color: #1677ff; }
.pagination span { font-size: 13px; color: #595959; min-width: 60px; text-align: center; }
</style>

<!-- 下拉面板全局样式（不加 scoped） -->
<style>
.col-dropdown {
  position: fixed;
  z-index: 9999;
  background: #fff;
  border: 1px solid #e0e0e0;
  border-radius: 6px;
  box-shadow: 0 6px 24px rgba(0,0,0,0.14);
  min-width: 240px;
  max-width: 420px;
  width: max-content;
  display: flex;
  flex-direction: column;
}
.cd-search { padding: 10px 10px 6px; }
.cd-search-inp { width: 100%; padding: 6px 10px; border: 1px solid #d9d9d9; border-radius: 4px; font-size: 13px; outline: none; box-sizing: border-box; background: #f5f5f5; }
.cd-search-inp:focus { border-color: #1677ff; background: #fff; }
.cd-actions { display: flex; gap: 12px; padding: 2px 12px 6px; border-bottom: 1px solid #f0f0f0; align-items: center; }
.cd-link { font-size: 13px; color: #1677ff; cursor: pointer; font-weight: 500; }
.cd-link:hover { text-decoration: underline; }
.cd-sort { font-size: 12px; color: #8c8c8c; cursor: pointer; padding: 1px 6px; border-radius: 3px; border: 1px solid #e8e8e8; transition: all 0.15s; margin-left: auto; }
.cd-sort:first-of-type { margin-left: auto; }
.cd-sort + .cd-sort { margin-left: 4px; }
.cd-sort:hover { border-color: #1677ff; color: #1677ff; }
.cd-sort.active { border-color: #1677ff; color: #1677ff; background: #e6f4ff; }
.cd-list { max-height: 260px; overflow-y: auto; padding: 4px 0; }
.cd-list::-webkit-scrollbar { width: 6px; }
.cd-list::-webkit-scrollbar-thumb { background: #d9d9d9; border-radius: 3px; }
.cd-item { display: flex; align-items: flex-start; gap: 10px; padding: 7px 14px; cursor: pointer; font-size: 13px; color: #262626; }
.cd-item:hover { background: #e6f4ff; }
.cd-item input[type=checkbox] { margin-top: 2px; flex-shrink: 0; accent-color: #1677ff; width: 15px; height: 15px; cursor: pointer; }
.cd-item-text { line-height: 1.5; word-break: break-all; flex: 1; }
.cd-empty-val { color: #aaa; font-style: italic; }
.cd-item-count { flex-shrink: 0; font-size: 12px; color: #aaa; margin-top: 2px; align-self: flex-start; }
.cd-empty { padding: 16px; text-align: center; color: #bfbfbf; font-size: 13px; }
.cd-footer { display: flex; gap: 8px; padding: 8px 12px; justify-content: flex-end; border-top: 1px solid #f0f0f0; }
.cd-btn-ok { padding: 5px 20px; background: #1677ff; border: none; border-radius: 4px; color: #fff; font-size: 13px; cursor: pointer; font-weight: 500; }
.cd-btn-ok:hover { background: #4096ff; }
.cd-btn-cancel { padding: 5px 14px; background: #fff; border: 1px solid #d9d9d9; border-radius: 4px; color: #595959; font-size: 13px; cursor: pointer; }
.cd-btn-cancel:hover { border-color: #1677ff; color: #1677ff; }
</style>
