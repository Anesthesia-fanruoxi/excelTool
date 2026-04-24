<script setup lang="ts">
import { ref, watch, onMounted, computed } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import type { TabData, PageResult } from '../types';

const props = defineProps<{ tab: TabData }>();

const PAGE_SIZE = 50;

const columns    = ref<string[]>([]);
const rows       = ref<(string | null)[][]>([]);
const rowIds     = ref<number[]>([]);
const total      = ref(0);
const curPage    = ref(1);
const isLoading  = ref(false);

const keyword    = ref('');
const searchCol  = ref('');
const applied    = ref({ keyword: '', col: '' });

const editing    = ref<{ rowIdx: number; colIdx: number } | null>(null);
const editVal    = ref('');
const totalPages = ref(1);

// 公式列索引集合（表头带 [公式] 后缀的列）
const formulaCols = computed(() => {
  const set = new Set<number>();
  columns.value.forEach((col, idx) => {
    if (col.endsWith('[公式]')) set.add(idx);
  });
  return set;
});

async function loadPage() {
  if (!props.tab.tableName) return;
  isLoading.value = true;
  try {
    const [result, ids] = await Promise.all([
      invoke<PageResult>('query_page', {
        tableName: props.tab.tableName,
        page: curPage.value,
        pageSize: PAGE_SIZE,
        keyword: applied.value.keyword,
        searchCol: applied.value.col,
      }),
      invoke<number[]>('get_row_ids', {
        tableName: props.tab.tableName,
        page: curPage.value,
        pageSize: PAGE_SIZE,
        keyword: applied.value.keyword,
        searchCol: applied.value.col,
      }),
    ]);
    columns.value    = result.columns;
    rows.value       = result.rows;
    rowIds.value     = ids;
    total.value      = result.total;
    totalPages.value = Math.max(1, Math.ceil(result.total / PAGE_SIZE));
    if (!searchCol.value && result.columns.length > 0) {
      searchCol.value = result.columns[0];
    }
  } catch (e) {
    console.error(e);
  } finally {
    isLoading.value = false;
  }
}

function onSearch() {
  applied.value = { keyword: keyword.value, col: searchCol.value };
  curPage.value = 1;
  loadPage();
}

function onReset() {
  keyword.value = '';
  applied.value = { keyword: '', col: '' };
  curPage.value = 1;
  loadPage();
}

function startEdit(rowIdx: number, colIdx: number) {
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
    await invoke('update_cell', {
      tableName: props.tab.tableName,
      rowId,
      column: col,
      value: val,
    });
    rows.value[rowIdx][colIdx] = val;
  } catch (e) {
    alert(`保存失败: ${e}`);
  }
}

function cancelEdit() { editing.value = null; }

// 表头显示名（去掉 [公式] 后缀）
function displayCol(col: string) {
  return col.endsWith('[公式]') ? col.slice(0, -4) : col;
}

onMounted(loadPage);
watch(() => props.tab.tableName, loadPage);
</script>

<template>
  <div class="viewer">
    <div class="toolbar">
      <select v-model="searchCol" class="col-select">
        <option v-for="c in columns" :key="c" :value="c">{{ displayCol(c) }}</option>
      </select>
      <input v-model="keyword" class="search-inp" placeholder="搜索..." @keydown.enter="onSearch" />
      <button class="btn-search" @click="onSearch">搜索</button>
      <button class="btn-reset" @click="onReset">重置</button>
      <span class="stat">共 <b>{{ total }}</b> 行</span>
    </div>

    <div class="table-wrap">
      <div v-if="isLoading" class="loading">加载中...</div>
      <div v-else-if="rows.length === 0" class="empty">暂无数据</div>
      <table v-else class="data-table">
        <thead>
          <tr>
            <th class="col-seq">#</th>
            <th v-for="col in columns" :key="col" :class="{ 'th-formula': col.endsWith('[公式]') }">
              {{ displayCol(col) }}
              <span v-if="col.endsWith('[公式]')" class="formula-badge">fx</span>
            </th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="(row, rIdx) in rows" :key="rowIds[rIdx]">
            <td class="col-seq">{{ (curPage - 1) * PAGE_SIZE + rIdx + 1 }}</td>
            <td
              v-for="(cell, cIdx) in row"
              :key="cIdx"
              class="cell"
              :class="{ 'cell-formula': formulaCols.has(cIdx) }"
              @dblclick="startEdit(rIdx, cIdx)"
            >
              <template v-if="editing && editing.rowIdx === rIdx && editing.colIdx === cIdx">
                <input
                  v-model="editVal"
                  class="cell-inp"
                  autofocus
                  @blur="commitEdit"
                  @keydown.enter="commitEdit"
                  @keydown.esc="cancelEdit"
                />
              </template>
              <template v-else>{{ cell ?? '' }}</template>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <div class="pagination">
      <button :disabled="curPage <= 1" @click="curPage--; loadPage()">上一页</button>
      <span>{{ curPage }} / {{ totalPages }}</span>
      <button :disabled="curPage >= totalPages" @click="curPage++; loadPage()">下一页</button>
    </div>
  </div>
</template>

<style scoped>
.viewer { display: flex; flex-direction: column; height: 100%; overflow: hidden; background: #f0f2f5; }

.toolbar { display: flex; align-items: center; gap: 8px; padding: 8px 12px; background: #fff; border-bottom: 1px solid #e8e8e8; flex-shrink: 0; }
.col-select { padding: 4px 8px; border: 1px solid #d9d9d9; border-radius: 4px; font-size: 13px; color: #333; max-width: 160px; }
.col-select:focus { outline: none; border-color: #1677ff; }
.search-inp { padding: 5px 10px; border: 1px solid #d9d9d9; border-radius: 4px; font-size: 13px; width: 200px; }
.search-inp:focus { outline: none; border-color: #1677ff; }
.btn-search { padding: 5px 12px; background: #1677ff; border: none; border-radius: 4px; color: #fff; font-size: 13px; cursor: pointer; }
.btn-search:hover { background: #4096ff; }
.btn-reset { padding: 5px 12px; background: #fff; border: 1px solid #d9d9d9; border-radius: 4px; color: #595959; font-size: 13px; cursor: pointer; }
.btn-reset:hover { border-color: #1677ff; color: #1677ff; }
.stat { font-size: 12px; color: #8c8c8c; margin-left: 4px; }
.stat b { color: #1677ff; }

.table-wrap { flex: 1; overflow: auto; background: #fff; }
.loading, .empty { display: flex; align-items: center; justify-content: center; height: 100%; color: #bfbfbf; font-size: 14px; }

.data-table { width: 100%; border-collapse: collapse; font-size: 13px; }
.data-table th, .data-table td { padding: 6px 10px; border-bottom: 1px solid #f0f0f0; border-right: 1px solid #f0f0f0; text-align: left; white-space: nowrap; }
.data-table th:last-child, .data-table td:last-child { border-right: none; }
.data-table thead th { background: #fafafa; color: #595959; font-weight: 600; position: sticky; top: 0; z-index: 10; border-bottom: 1px solid #e8e8e8; }
.data-table tbody tr:hover td { background: #e6f4ff; }
.col-seq { width: 46px; text-align: center; color: #bfbfbf !important; background: #fafafa !important; font-size: 12px; }
.cell { cursor: default; min-width: 80px; max-width: 300px; overflow: hidden; text-overflow: ellipsis; }
.cell-formula { background: #f6ffed !important; }
.th-formula { color: #389e0d !important; }
.formula-badge { display: inline-block; margin-left: 4px; padding: 0 3px; background: #389e0d; color: #fff; border-radius: 3px; font-size: 10px; font-family: monospace; vertical-align: middle; }
.cell-inp { width: 100%; padding: 2px 4px; border: 2px solid #1677ff; border-radius: 2px; font-size: 13px; outline: none; background: #fff; }

.pagination { display: flex; align-items: center; justify-content: center; gap: 12px; padding: 8px; background: #fff; border-top: 1px solid #f0f0f0; flex-shrink: 0; }
.pagination button { padding: 4px 14px; background: #fff; border: 1px solid #d9d9d9; border-radius: 4px; color: #595959; font-size: 13px; cursor: pointer; }
.pagination button:disabled { opacity: 0.4; cursor: not-allowed; }
.pagination button:hover:not(:disabled) { border-color: #1677ff; color: #1677ff; }
.pagination span { font-size: 13px; color: #595959; min-width: 60px; text-align: center; }
</style>
