<script setup lang="ts">
import { useSalesView } from '../composables/useSalesView';
import { useCopyCell } from '../composables/useCopyCell';
import { useExport } from '../composables/useExport';
import SalesDetailModal from '../components/sales/SalesDetailModal.vue';
import SalesEditModal from '../components/sales/SalesEditModal.vue';
import SalesDeleteConfirm from '../components/sales/SalesDeleteConfirm.vue';

const {
  LIST_COLUMNS,
  isLoading,
  isSaving,
  rows,
  total,
  searchText,
  searchCol,
  curPage,
  totalPages,
  detailRow,
  dialog,
  deleteConfirm,
  toastMsg,
  toastType,
  onSearch,
  openDetail,
  closeDetail,
  openAdd,
  openEdit,
  saveDialog,
  confirmDelete,
  doDelete,
} = useSalesView();

const { copiedKey, toastVisible, copyCell } = useCopyCell();
const { isExporting, exportProgress, exportXlsx } = useExport();

function exportAll() {
  exportXlsx('', '', '销售表_全部');
}

function exportFiltered() {
  const name = searchText.value
    ? `销售表_筛选_${searchText.value}`
    : '销售表_全部';
  exportXlsx(searchText.value, '', name);
}
</script>

<template>
  <div class="sales-view">
    <!-- Toast -->
    <transition name="fade">
      <div v-if="toastMsg" :class="['toast', toastType === 'error' ? 'toast-error' : 'toast-success']">
        {{ toastMsg }}
      </div>
    </transition>

    <!-- 复制成功提示 -->
    <transition name="slide-up">
      <div v-if="toastVisible" class="copy-toast">复制成功 ✓</div>
    </transition>

    <!-- 顶部栏 -->
    <div class="top-bar">
      <span class="page-title">销售表</span>
      <div class="top-actions">
        <select v-model="searchCol" class="sel" @change="onSearch">
          <option value="">全部列</option>
          <option v-for="c in LIST_COLUMNS" :key="c" :value="c">{{ c }}</option>
        </select>
        <input v-model="searchText" class="search-input" placeholder="搜索..." @input="onSearch" />
        <div class="export-group">
          <button class="btn-export" :disabled="isExporting || total === 0" @click="exportAll">
            {{ isExporting ? `导出中 ${exportProgress}%` : '导出全部' }}
          </button>
          <button
            v-if="searchText"
            class="btn-export btn-export-filter"
            :disabled="isExporting"
            @click="exportFiltered"
          >
            导出筛选 ({{ total }})
          </button>
        </div>
        <button class="btn-add" @click="openAdd">+ 新增</button>
      </div>
    </div>

    <!-- 统计栏 -->
    <div class="stat-bar">
      共 <b>{{ total }}</b> 条
      <span v-if="isSaving" class="saving-tip">保存中...</span>
    </div>

    <!-- 表格 -->
    <div class="table-wrap">
      <div v-if="isLoading" class="status-center">加载中...</div>
      <div v-else-if="rows.length === 0" class="status-center">
        <div class="empty-icon">📋</div>
        <p>暂无数据，请先在「数据管理」中导入</p>
      </div>
      <table v-else class="data-table">
        <thead>
          <tr>
            <th class="col-seq">#</th>
            <th v-for="col in LIST_COLUMNS" :key="col">{{ col }}</th>
            <th class="col-op">操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="(row, idx) in rows" :key="idx">
            <td class="col-seq">{{ (curPage - 1) * 50 + idx + 1 }}</td>
            <td
              v-for="col in LIST_COLUMNS"
              :key="col"
              :class="{ 'cell-copied': copiedKey === `${idx}-${col}` }"
              @dblclick="copyCell(`${idx}-${col}`, row[col] ?? '')"
            >{{ row[col] ?? '' }}</td>
            <td class="col-op">
              <button class="btn-detail" @click="openDetail(row)">详情</button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- 分页 -->
    <div v-if="total > 0" class="pagination">
      <button :disabled="curPage === 1" @click="curPage--">上一页</button>
      <span>{{ curPage }} / {{ totalPages }}</span>
      <button :disabled="curPage >= totalPages" @click="curPage++">下一页</button>
    </div>

    <!-- 详情弹框 -->
    <SalesDetailModal
      v-if="detailRow"
      :row="detailRow"
      @close="closeDetail"
      @edit="openEdit"
      @delete="confirmDelete"
    />

    <!-- 编辑弹框 -->
    <SalesEditModal
      v-if="dialog.show"
      :mode="dialog.mode"
      :form="dialog.form"
      :is-saving="isSaving"
      @close="dialog.show = false"
      @save="saveDialog"
    />

    <!-- 删除确认 -->
    <SalesDeleteConfirm
      v-if="deleteConfirm.show"
      @cancel="deleteConfirm.show = false"
      @confirm="doDelete"
    />
  </div>
</template>

<style scoped>
.sales-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.toast {
  position: fixed; top: 20px; right: 20px;
  padding: 10px 18px; border-radius: 6px; font-size: 13px;
  z-index: 9999; box-shadow: 0 4px 16px rgba(0,0,0,0.4);
}
.toast-error   { background: #e74c3c; color: #fff; }
.toast-success { background: #27ae60; color: #fff; }
.fade-enter-active, .fade-leave-active { transition: opacity 0.3s; }
.fade-enter-from, .fade-leave-to { opacity: 0; }

.copy-toast {
  position: fixed;
  bottom: 40px;
  left: 50%;
  transform: translateX(-50%);
  background: #27ae60;
  color: #fff;
  padding: 8px 24px;
  border-radius: 20px;
  font-size: 13px;
  z-index: 9999;
  box-shadow: 0 4px 12px rgba(0,0,0,0.3);
  pointer-events: none;
}
.slide-up-enter-active, .slide-up-leave-active { transition: all 0.25s ease; }
.slide-up-enter-from, .slide-up-leave-to { opacity: 0; transform: translateX(-50%) translateY(10px); }

.top-bar {
  display: flex; align-items: center; justify-content: space-between;
  padding: 0 20px; height: 56px; border-bottom: 1px solid #0f3460; flex-shrink: 0;
}
.page-title { font-size: 17px; font-weight: 600; color: #fff; }
.top-actions { display: flex; align-items: center; gap: 8px; }

.sel {
  padding: 6px 10px; background: #0f3460; border: 1px solid #1a4a7a;
  border-radius: 4px; color: #ccc; font-size: 13px; cursor: pointer;
}
.sel:focus { outline: none; border-color: #4a9eff; }

.search-input {
  padding: 6px 12px; background: #0f3460; border: 1px solid #1a4a7a;
  border-radius: 4px; color: #fff; font-size: 13px; width: 200px;
}
.search-input:focus { outline: none; border-color: #4a9eff; }

.btn-add {
  padding: 7px 16px; background: #4a9eff; border: none; border-radius: 6px;
  color: #fff; font-size: 13px; cursor: pointer; transition: background 0.2s;
}
.btn-add:hover { background: #3a8eef; }

.export-group { display: flex; gap: 6px; }
.btn-export {
  padding: 7px 14px; background: #0f3460; border: 1px solid #1a4a7a;
  border-radius: 6px; color: #ccc; font-size: 13px;
  cursor: pointer; transition: all 0.2s; white-space: nowrap;
}
.btn-export:hover:not(:disabled) { background: #27ae60; border-color: #27ae60; color: #fff; }
.btn-export:disabled { opacity: 0.5; cursor: not-allowed; }
.btn-export-filter { border-color: #4a9eff44; color: #4a9eff; }

.stat-bar {
  padding: 6px 20px; font-size: 12px; color: #666;
  border-bottom: 1px solid #0f3460; flex-shrink: 0;
}
.stat-bar b { color: #4a9eff; }
.saving-tip { margin-left: 12px; color: #4a9eff; }

.table-wrap { flex: 1; overflow: auto; }

.status-center {
  display: flex; flex-direction: column; align-items: center; justify-content: center;
  height: 100%; color: #555; gap: 12px; font-size: 14px;
}
.empty-icon { font-size: 40px; }

.data-table {
  width: 100%; border-collapse: collapse; font-size: 13px;
}
.data-table th,
.data-table td {
  padding: 10px 12px; border: 1px solid #0f3460; text-align: left;
}
.data-table thead th {
  background: #16213e; color: #4a9eff; font-weight: 600;
  position: sticky; top: 0; z-index: 10; white-space: nowrap;
}
.data-table tbody td { color: #ccc; background: #1a1a2e; }
.data-table tbody tr:hover td { background: #16213e; }
.cell-copied { color: #27ae60 !important; background: #0d2a1a !important; }

.col-seq {
  width: 50px; text-align: center;
  background: #0f3460 !important; color: #555 !important;
}
.col-op {
  width: 80px; text-align: center;
}
.btn-detail {
  padding: 4px 12px; background: #0f3460; border: none; border-radius: 4px;
  color: #ccc; font-size: 12px; cursor: pointer; transition: all 0.2s;
}
.btn-detail:hover { background: #4a9eff; color: #fff; }

.pagination {
  display: flex; align-items: center; justify-content: center;
  gap: 14px; padding: 10px; border-top: 1px solid #0f3460; flex-shrink: 0;
}
.pagination button {
  padding: 5px 14px; background: #0f3460; border: none; border-radius: 4px;
  color: #ccc; font-size: 13px; cursor: pointer; transition: all 0.2s;
}
.pagination button:disabled { opacity: 0.4; cursor: not-allowed; }
.pagination button:hover:not(:disabled) { background: #4a9eff; color: #fff; }
.pagination span { color: #ccc; font-size: 13px; min-width: 60px; text-align: center; }
</style>
