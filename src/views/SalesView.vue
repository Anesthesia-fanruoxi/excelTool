<script setup lang="ts">
import { useSalesView } from '../composables/useSalesView';
import { useCopyCell } from '../composables/useCopyCell';
import { useNavigationStore } from '../stores/navigation';

const nav = useNavigationStore();

const {
  SALES_LIST_FIELDS, SALES_FILTER_FIELDS, SALES_FIELD_LABELS,
  isLoading, rows, total, curPage, pageSize, totalPages,
  filterField, filterKw,
  toastMsg, toastType,
  onSearch, onReset, prevPage, nextPage,
} = useSalesView();

const { copiedKey, toastVisible, copyCell } = useCopyCell();
</script>

<template>
  <div class="view-wrap">
    <transition name="fade">
      <div v-if="toastMsg" :class="['toast', toastType === 'error' ? 'toast-error' : 'toast-success']">{{ toastMsg }}</div>
    </transition>
    <transition name="slide-up">
      <div v-if="toastVisible" class="copy-toast">复制成功 ✓</div>
    </transition>

    <div class="top-bar">
      <span class="page-title">销售明细</span>
      <div class="top-actions">
        <select v-model="filterField" class="sel">
          <option v-for="f in SALES_FILTER_FIELDS" :key="f.field" :value="f.field">{{ f.label }}</option>
        </select>
        <input v-model="filterKw" class="search-input" placeholder="搜索..." @keydown.enter="onSearch" />
        <button class="btn-search" @click="onSearch">搜索</button>
        <button class="btn-reset" @click="onReset">重置</button>
      </div>
    </div>

    <div class="stat-bar">共 <b>{{ total }}</b> 条</div>

    <div class="table-wrap">
      <div v-if="isLoading" class="status-center">加载中...</div>
      <div v-else-if="rows.length === 0" class="status-center">
        <div class="empty-icon">📋</div>
        <p>暂无数据，请先在「数据管理」中导入销售表</p>
      </div>
      <table v-else class="data-table">
        <thead>
          <tr>
            <th class="col-seq">#</th>
            <th v-for="f in SALES_LIST_FIELDS" :key="f">{{ SALES_FIELD_LABELS[f] }}</th>
            <th class="col-op">操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="(row, idx) in rows" :key="row.id">
            <td class="col-seq">{{ (curPage - 1) * pageSize + idx + 1 }}</td>
            <td
              v-for="f in SALES_LIST_FIELDS" :key="f"
              :class="{ 'cell-copied': copiedKey === `${idx}-${f}` }"
              @dblclick="copyCell(`${idx}-${f}`, String(row[f] ?? ''))"
            >
              <template v-if="f === 'unit_price' || f === 'quantity'">
                {{ row[f] > 0 ? row[f] : '' }}
              </template>
              <template v-else>{{ row[f] ?? '' }}</template>
            </td>
            <td class="col-op">
              <button
                v-if="row.contract_no"
                class="btn-contract"
                @click="nav.navigateToContract(row.contract_no)"
              >查看合同</button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <div v-if="total > 0" class="pagination">
      <button :disabled="curPage === 1" @click="prevPage">上一页</button>
      <span>{{ curPage }} / {{ totalPages }}</span>
      <button :disabled="curPage >= totalPages" @click="nextPage">下一页</button>
    </div>
  </div>
</template>

<style scoped>
.view-wrap { display: flex; flex-direction: column; height: 100%; overflow: hidden; background: #f0f2f5; }
.toast { position: fixed; top: 20px; right: 20px; padding: 10px 18px; border-radius: 6px; font-size: 13px; z-index: 9999; box-shadow: 0 4px 16px rgba(0,0,0,0.15); }
.toast-error { background: #ff4d4f; color: #fff; }
.toast-success { background: #52c41a; color: #fff; }
.fade-enter-active, .fade-leave-active { transition: opacity 0.3s; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
.copy-toast { position: fixed; bottom: 40px; left: 50%; transform: translateX(-50%); background: #52c41a; color: #fff; padding: 6px 20px; border-radius: 20px; font-size: 13px; z-index: 9999; pointer-events: none; }
.slide-up-enter-active, .slide-up-leave-active { transition: all 0.25s ease; }
.slide-up-enter-from, .slide-up-leave-to { opacity: 0; transform: translateX(-50%) translateY(10px); }
.top-bar { display: flex; align-items: center; justify-content: space-between; padding: 0 16px; height: 52px; background: #fff; border-bottom: 1px solid #e8e8e8; flex-shrink: 0; }
.page-title { font-size: 16px; font-weight: 600; color: #262626; }
.top-actions { display: flex; align-items: center; gap: 8px; }
.sel { padding: 5px 8px; background: #fff; border: 1px solid #d9d9d9; border-radius: 4px; color: #333; font-size: 13px; cursor: pointer; }
.sel:focus { outline: none; border-color: #1677ff; }
.search-input { padding: 5px 10px; background: #fff; border: 1px solid #d9d9d9; border-radius: 4px; color: #333; font-size: 13px; width: 180px; }
.search-input:focus { outline: none; border-color: #1677ff; }
.btn-search { padding: 5px 12px; background: #1677ff; border: none; border-radius: 4px; color: #fff; font-size: 13px; cursor: pointer; }
.btn-search:hover { background: #4096ff; }
.btn-reset { padding: 5px 12px; background: #fff; border: 1px solid #d9d9d9; border-radius: 4px; color: #595959; font-size: 13px; cursor: pointer; }
.btn-reset:hover { border-color: #1677ff; color: #1677ff; }
.stat-bar { padding: 4px 16px; font-size: 12px; color: #8c8c8c; background: #fafafa; border-bottom: 1px solid #f0f0f0; flex-shrink: 0; }
.stat-bar b { color: #1677ff; }
.table-wrap { flex: 1; overflow: auto; background: #fff; }
.status-center { display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100%; color: #bfbfbf; gap: 12px; font-size: 14px; }
.empty-icon { font-size: 40px; }
.data-table { width: 100%; border-collapse: collapse; font-size: 13px; }
.data-table th, .data-table td { padding: 6px 10px; border-bottom: 1px solid #f0f0f0; border-right: 1px solid #f0f0f0; text-align: left; white-space: nowrap; }
.data-table th:last-child, .data-table td:last-child { border-right: none; }
.data-table thead th { background: #fafafa; color: #595959; font-weight: 600; position: sticky; top: 0; z-index: 10; border-bottom: 1px solid #e8e8e8; }
.data-table tbody td { color: #333; background: #fff; }
.data-table tbody tr:hover td { background: #e6f4ff; }
.cell-copied { color: #52c41a !important; background: #f6ffed !important; }
.col-seq { width: 46px; text-align: center; color: #bfbfbf !important; background: #fafafa !important; }
.col-op { width: 90px; text-align: center; }
.btn-contract { padding: 2px 8px; background: #fff; border: 1px solid #d9d9d9; border-radius: 4px; color: #722ed1; font-size: 12px; cursor: pointer; transition: all 0.15s; white-space: nowrap; }
.btn-contract:hover { background: #722ed1; color: #fff; border-color: #722ed1; }
.col-link { width: 80px; text-align: center; }
.tag-linked { display: inline-block; padding: 1px 8px; border-radius: 10px; font-size: 11px; background: #f6ffed; color: #52c41a; border: 1px solid #b7eb8f; }
.tag-unlinked { display: inline-block; padding: 1px 8px; border-radius: 10px; font-size: 11px; background: #fff2f0; color: #ff4d4f; border: 1px solid #ffccc7; }
.pagination { display: flex; align-items: center; justify-content: center; gap: 12px; padding: 8px; background: #fff; border-top: 1px solid #f0f0f0; flex-shrink: 0; }
.pagination button { padding: 4px 12px; background: #fff; border: 1px solid #d9d9d9; border-radius: 4px; color: #595959; font-size: 13px; cursor: pointer; }
.pagination button:disabled { opacity: 0.4; cursor: not-allowed; }
.pagination button:hover:not(:disabled) { border-color: #1677ff; color: #1677ff; }
.pagination span { color: #595959; font-size: 13px; min-width: 60px; text-align: center; }
</style>
