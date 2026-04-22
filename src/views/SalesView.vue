<script setup lang="ts">
import { useSalesView } from '../composables/useSalesView';
import { useCopyCell } from '../composables/useCopyCell';
import { useExport } from '../composables/useExport';
import SalesDetailModal from '../components/sales/SalesDetailModal.vue';
import SalesEditModal from '../components/sales/SalesEditModal.vue';
import SalesDeleteConfirm from '../components/sales/SalesDeleteConfirm.vue';

const {
  LIST_COLUMNS, isLoading, isSaving, rows, total,
  searchText, searchCol, curPage, totalPages,
  detailRow, dialog, deleteConfirm, toastMsg, toastType,
  onSearch, openDetail, closeDetail, openAdd, openEdit,
  saveDialog, confirmDelete, doDelete,
} = useSalesView();

const { copiedKey, toastVisible, copyCell } = useCopyCell();
const { isExporting, exportProgress, exportXlsx } = useExport();

function exportAll() { exportXlsx('', '', '销售表_全部'); }
function exportFiltered() {
  exportXlsx(searchText.value, '', searchText.value ? `销售表_筛选_${searchText.value}` : '销售表_全部');
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
          <button v-if="searchText" class="btn-export btn-export-filter" :disabled="isExporting" @click="exportFiltered">
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
              v-for="col in LIST_COLUMNS" :key="col"
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

    <SalesDetailModal v-if="detailRow" :row="detailRow" @close="closeDetail" @edit="openEdit" @delete="confirmDelete" />
    <SalesEditModal v-if="dialog.show" :mode="dialog.mode" :form="dialog.form" :is-saving="isSaving" @close="dialog.show = false" @save="saveDialog" />
    <SalesDeleteConfirm v-if="deleteConfirm.show" @cancel="deleteConfirm.show = false" @confirm="doDelete" />
  </div>
</template>

<style scoped>
.sales-view { display: flex; flex-direction: column; height: 100%; overflow: hidden; background: #f0f2f5; }

/* stylelint-disable selector-pseudo-class-no-unknown */
.toast { position: fixed; top: 20px; right: 20px; padding: 10px 18px; border-radius: 6px; font-size: 13px; z-index: 9999; box-shadow: 0 4px 16px rgba(0,0,0,0.15); }
.toast-error   { background: #ff4d4f; color: #fff; }
.toast-success { background: #52c41a; color: #fff; }
.fade-enter-active, .fade-leave-active { transition: opacity 0.3s; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
.copy-toast { position: fixed; bottom: 40px; left: 50%; transform: translateX(-50%); background: #52c41a; color: #fff; padding: 6px 20px; border-radius: 20px; font-size: 13px; z-index: 9999; pointer-events: none; }
.slide-up-enter-active, .slide-up-leave-active { transition: all 0.25s ease; }
.slide-up-enter-from, .slide-up-leave-to { opacity: 0; transform: translateX(-50%) translateY(10px); }
/* stylelint-enable */

.top-bar { display: flex; align-items: center; justify-content: space-between; padding: 0 16px; height: 52px; background: #fff; border-bottom: 1px solid #e8e8e8; flex-shrink: 0; }
.page-title { font-size: 16px; font-weight: 600; color: #262626; }
.top-actions { display: flex; align-items: center; gap: 8px; }

.sel { padding: 5px 8px; background: #fff; border: 1px solid #d9d9d9; border-radius: 4px; color: #333; font-size: 13px; cursor: pointer; }
.sel:focus { outline: none; border-color: #1677ff; }
.search-input { padding: 5px 10px; background: #fff; border: 1px solid #d9d9d9; border-radius: 4px; color: #333; font-size: 13px; width: 180px; }
.search-input:focus { outline: none; border-color: #1677ff; box-shadow: 0 0 0 2px rgba(22,119,255,0.1); }

.export-group { display: flex; gap: 6px; }
.btn-export { padding: 5px 12px; background: #fff; border: 1px solid #d9d9d9; border-radius: 4px; color: #595959; font-size: 13px; cursor: pointer; transition: all 0.2s; white-space: nowrap; }
.btn-export:hover:not(:disabled) { border-color: #52c41a; color: #52c41a; }
.btn-export:disabled { opacity: 0.5; cursor: not-allowed; }
.btn-export-filter { border-color: #1677ff44; color: #1677ff; }
.btn-add { padding: 5px 14px; background: #1677ff; border: none; border-radius: 4px; color: #fff; font-size: 13px; cursor: pointer; transition: background 0.2s; }
.btn-add:hover { background: #4096ff; }

.stat-bar { padding: 4px 16px; font-size: 12px; color: #8c8c8c; background: #fafafa; border-bottom: 1px solid #f0f0f0; flex-shrink: 0; }
.stat-bar b { color: #1677ff; }
.saving-tip { margin-left: 12px; color: #1677ff; }

.table-wrap { flex: 1; overflow: auto; background: #fff; }
.status-center { display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100%; color: #bfbfbf; gap: 12px; font-size: 14px; }
.empty-icon { font-size: 40px; }

.data-table { width: 100%; border-collapse: collapse; font-size: 13px; }
.data-table th, .data-table td { padding: 6px 10px; border-bottom: 1px solid #f0f0f0; text-align: left; white-space: nowrap; }
.data-table thead th { background: #fafafa; color: #595959; font-weight: 600; position: sticky; top: 0; z-index: 10; border-bottom: 1px solid #e8e8e8; }
.data-table tbody td { color: #333; background: #fff; }
.data-table tbody tr:hover td { background: #e6f4ff; }
.cell-copied { color: #52c41a !important; background: #f6ffed !important; }
.col-seq { width: 46px; text-align: center; color: #bfbfbf !important; background: #fafafa !important; }
.col-op { width: 70px; text-align: center; }
.btn-detail { padding: 2px 10px; background: #fff; border: 1px solid #d9d9d9; border-radius: 4px; color: #595959; font-size: 12px; cursor: pointer; transition: all 0.2s; }
.btn-detail:hover { border-color: #1677ff; color: #1677ff; }

.pagination { display: flex; align-items: center; justify-content: center; gap: 12px; padding: 8px; background: #fff; border-top: 1px solid #f0f0f0; flex-shrink: 0; }
.pagination button { padding: 4px 12px; background: #fff; border: 1px solid #d9d9d9; border-radius: 4px; color: #595959; font-size: 13px; cursor: pointer; transition: all 0.2s; }
.pagination button:disabled { opacity: 0.4; cursor: not-allowed; }
.pagination button:hover:not(:disabled) { border-color: #1677ff; color: #1677ff; }
.pagination span { color: #595959; font-size: 13px; min-width: 60px; text-align: center; }
</style>
