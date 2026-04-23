<script setup lang="ts">
import { useSalesView } from '../composables/useSalesView';
import { useCopyCell } from '../composables/useCopyCell';
import { useExport } from '../composables/useExport';
import SalesDetailModal from '../components/sales/SalesDetailModal.vue';
import SalesEditModal from '../components/sales/SalesEditModal.vue';
import SalesDeleteConfirm from '../components/sales/SalesDeleteConfirm.vue';
import { useNavigationStore } from '../stores/navigation';

const nav = useNavigationStore();

const {
  LIST_COLUMNS, FILTER_COLUMNS, isLoading, isSaving, rows, total,
  conditions, curPage, pageSize, totalPages,
  detailRow, dialog, deleteConfirm, toastMsg, toastType,
  onSearch, addCondition, removeCondition, resetConditions,
  prevPage, nextPage,
  openDetail, closeDetail, openEdit,
  saveDialog, confirmDelete, doDelete,
} = useSalesView();

const { copiedKey, toastVisible, copyCell } = useCopyCell();
const { isExporting, exportProgress, exportXlsx } = useExport();

function getActiveConditions(): [string, string][] {
  return conditions.value
    .filter(c => c.kw.trim() !== '')
    .map(c => [c.col, c.kw.trim()] as [string, string]);
}
function exportAll() { exportXlsx([], '', '销售表_全部'); }
function exportFiltered() {
  const conds = getActiveConditions();
  exportXlsx(conds, '', conds.length ? '销售表_筛选' : '销售表_全部');
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
      <span class="page-title">销售明细</span>
      <div class="top-actions">
        <div class="export-group">
          <button class="btn-export" :disabled="isExporting || total === 0" @click="exportAll">
            {{ isExporting ? `导出中 ${exportProgress}%` : '导出全部' }}
          </button>
          <button v-if="getActiveConditions().length" class="btn-export btn-export-filter" :disabled="isExporting" @click="exportFiltered">
            导出筛选 ({{ total }})
          </button>
        </div>
      </div>
    </div>

    <!-- 搜索条件栏 -->
    <div class="search-bar">
      <div class="conditions">
        <div v-for="(cond, idx) in conditions" :key="idx" class="condition-row">
          <select v-model="cond.col" class="sel">
            <option value="">全部列</option>
            <option v-for="c in FILTER_COLUMNS" :key="c" :value="c">{{ c }}</option>
          </select>
          <input
            v-model="cond.kw"
            class="search-input"
            :placeholder="cond.col ? `搜索 ${cond.col}...` : '关键词...'"
            @keydown.enter="onSearch"
          />
          <button class="btn-remove-cond" :disabled="conditions.length === 1 && !cond.kw && !cond.col" @click="removeCondition(idx)">×</button>
        </div>
      </div>
      <div class="search-actions">
        <button class="btn-add-cond" @click="addCondition">+ 添加条件</button>
        <button class="btn-reset" @click="resetConditions">重置</button>
        <button class="btn-search" @click="onSearch">🔍 搜索</button>
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
            <td class="col-seq">{{ (curPage - 1) * pageSize + idx + 1 }}</td>
            <td
              v-for="col in LIST_COLUMNS" :key="col"
              :class="{ 'cell-copied': copiedKey === `${idx}-${col}` }"
              @dblclick="copyCell(`${idx}-${col}`, row[col] ?? '')"
            >
              <span v-if="col === '状态列' && row[col]" :class="['status-tag', `status-${row[col]}`]">{{ row[col] }}</span>
              <span v-else-if="col === '利润' && row[col]" :class="['profit-tag', parseFloat(row[col]) >= 0 ? 'profit-pos' : 'profit-neg']">{{ row[col] }}</span>
              <template v-else>{{ row[col] ?? '' }}</template>
            </td>
            <td class="col-op">
              <button class="btn-detail" @click="openDetail(row)">详情</button>
              <button v-if="row['合同号']" class="btn-goto-contract" @click="nav.navigateToContract(row['合同号'])">查看合同</button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- 分页 -->
    <div v-if="total > 0" class="pagination">
      <button :disabled="curPage === 1" @click="prevPage">上一页</button>
      <span>{{ curPage }} / {{ totalPages }}</span>
      <button :disabled="curPage >= totalPages" @click="nextPage">下一页</button>
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

.search-group { display: flex; align-items: center; gap: 6px; }
.sel { padding: 5px 8px; background: #fff; border: 1px solid #d9d9d9; border-radius: 4px; color: #333; font-size: 13px; cursor: pointer; }
.sel:focus { outline: none; border-color: #1677ff; }
.search-input { padding: 5px 10px; background: #fff; border: 1px solid #d9d9d9; border-radius: 4px; color: #333; font-size: 13px; width: 180px; }
.search-input:focus { outline: none; border-color: #1677ff; box-shadow: 0 0 0 2px rgba(22,119,255,0.1); }

/* 搜索条件栏 */
.search-bar { display: flex; align-items: flex-start; gap: 12px; padding: 10px 16px; background: #fafafa; border-bottom: 1px solid #e8e8e8; flex-shrink: 0; }
.conditions { display: flex; flex-direction: column; gap: 6px; flex: 1; }
.condition-row { display: flex; align-items: center; gap: 6px; }
.condition-row .sel { width: 130px; flex-shrink: 0; }
.condition-row .search-input { flex: 1; min-width: 0; }
.btn-remove-cond { width: 24px; height: 24px; border: 1px solid #d9d9d9; border-radius: 4px; background: #fff; color: #8c8c8c; font-size: 14px; cursor: pointer; line-height: 1; flex-shrink: 0; transition: all 0.2s; }
.btn-remove-cond:hover:not(:disabled) { border-color: #ff4d4f; color: #ff4d4f; }
.btn-remove-cond:disabled { opacity: 0.3; cursor: not-allowed; }
.search-actions { display: flex; flex-direction: column; gap: 6px; flex-shrink: 0; }
.btn-add-cond { padding: 5px 10px; background: #fff; border: 1px solid #d9d9d9; border-radius: 4px; color: #595959; font-size: 12px; cursor: pointer; white-space: nowrap; transition: all 0.2s; }
.btn-add-cond:hover { border-color: #1677ff; color: #1677ff; }
.btn-reset { padding: 5px 10px; background: #fff; border: 1px solid #d9d9d9; border-radius: 4px; color: #595959; font-size: 12px; cursor: pointer; white-space: nowrap; transition: all 0.2s; }
.btn-reset:hover { border-color: #ff4d4f; color: #ff4d4f; }
.btn-search { padding: 5px 14px; background: #1677ff; border: none; border-radius: 4px; color: #fff; font-size: 12px; cursor: pointer; white-space: nowrap; transition: background 0.2s; }
.btn-search:hover { background: #4096ff; }

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
.data-table th, .data-table td { padding: 6px 10px; border-bottom: 1px solid #f0f0f0; border-right: 1px solid #f0f0f0; text-align: left; white-space: nowrap; }
.data-table th:last-child, .data-table td:last-child { border-right: none; }
.data-table thead th { background: #fafafa; color: #595959; font-weight: 600; position: sticky; top: 0; z-index: 10; border-bottom: 1px solid #e8e8e8; border-right: 1px solid #e8e8e8; }
.data-table tbody td { color: #333; background: #fff; }
.data-table tbody tr:hover td { background: #e6f4ff; }
.cell-copied { color: #52c41a !important; background: #f6ffed !important; }

/* 状态列 tag */
.status-tag { display: inline-block; padding: 1px 8px; border-radius: 10px; font-size: 12px; font-weight: 500; white-space: nowrap; }
.status-等回签     { color: #ff4d4f; background: #fff2f0; border: 1px solid #ffccc7; }
.status-回签不完整  { color: #1677ff; background: #e6f4ff; border: 1px solid #91caff; }
.status-待对账     { color: #722ed1; background: #f9f0ff; border: 1px solid #d3adf7; }
.status-已对账     { color: #52c41a; background: #f6ffed; border: 1px solid #b7eb8f; }

/* 利润 tag */
.profit-tag { display: inline-block; padding: 1px 8px; border-radius: 10px; font-size: 12px; font-weight: 500; white-space: nowrap; }
.profit-pos { color: #52c41a; background: #f6ffed; border: 1px solid #b7eb8f; }
.profit-neg { color: #ff4d4f; background: #fff2f0; border: 1px solid #ffccc7; }
.col-seq { width: 46px; text-align: center; color: #bfbfbf !important; background: #fafafa !important; }
.col-op { width: 120px; text-align: center; }
.btn-detail { padding: 2px 10px; background: #fff; border: 1px solid #d9d9d9; border-radius: 4px; color: #595959; font-size: 12px; cursor: pointer; transition: all 0.2s; }
.btn-detail:hover { border-color: #1677ff; color: #1677ff; }
.btn-goto-contract { padding: 2px 8px; background: #fff; border: 1px solid #d9d9d9; border-radius: 4px; color: #722ed1; font-size: 12px; cursor: pointer; transition: all 0.2s; margin-left: 4px; }
.btn-goto-contract:hover { background: #722ed1; color: #fff; border-color: #722ed1; }

.pagination { display: flex; align-items: center; justify-content: center; gap: 12px; padding: 8px; background: #fff; border-top: 1px solid #f0f0f0; flex-shrink: 0; }
.pagination button { padding: 4px 12px; background: #fff; border: 1px solid #d9d9d9; border-radius: 4px; color: #595959; font-size: 13px; cursor: pointer; transition: all 0.2s; }
.pagination button:disabled { opacity: 0.4; cursor: not-allowed; }
.pagination button:hover:not(:disabled) { border-color: #1677ff; color: #1677ff; }
.pagination span { color: #595959; font-size: 13px; min-width: 60px; text-align: center; }
</style>
