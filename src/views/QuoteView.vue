<script setup lang="ts">
import { useQuoteView } from '../composables/useQuoteView';
import { useCopyCell } from '../composables/useCopyCell';

const {
  QUOTE_LIST_COLUMNS,
  isLoading,
  rows,
  total,
  curPage,
  totalPages,
  filterContractNo,
  filterGoodsName,
  filterSpec,
  filterSupplier,
  toastMsg,
  toastType,
  onSearch,
  onReset,
  prevPage,
  nextPage,
} = useQuoteView();

const { copiedKey, toastVisible, copyCell } = useCopyCell();

function profitClass(val: string) {
  const n = parseFloat(val);
  return isNaN(n) ? '' : n >= 0 ? 'profit-pos' : 'profit-neg';
}
</script>

<template>
  <div class="quote-view">
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
      <span class="page-title">报价明细</span>
      <div class="top-actions">
        <input
          v-model="filterContractNo"
          class="search-input search-input-sm"
          placeholder="合同号"
          @keydown.enter="onSearch"
        />
        <input
          v-model="filterGoodsName"
          class="search-input search-input-sm"
          placeholder="货物名称"
          @keydown.enter="onSearch"
        />
        <input
          v-model="filterSpec"
          class="search-input search-input-sm"
          placeholder="规格型号"
          @keydown.enter="onSearch"
        />
        <input
          v-model="filterSupplier"
          class="search-input search-input-sm"
          placeholder="供应商"
          @keydown.enter="onSearch"
        />
        <button class="btn-search" @click="onSearch">搜索</button>
        <button class="btn-reset" @click="onReset">重置</button>
      </div>
    </div>

    <!-- 统计栏 -->
    <div class="stat-bar">共 <b>{{ total }}</b> 条</div>

    <!-- 表格 -->
    <div class="table-wrap">
      <div v-if="isLoading" class="status-center">加载中...</div>
      <div v-else-if="rows.length === 0" class="status-center">
        <div class="empty-icon">💰</div>
        <p>暂无数据，请先在「数据管理」中导入报价表</p>
      </div>
      <table v-else class="data-table">
        <thead>
          <tr>
            <th class="col-seq">#</th>
            <th v-for="col in QUOTE_LIST_COLUMNS" :key="col">{{ col }}</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="(row, idx) in rows" :key="idx">
            <td class="col-seq">{{ (curPage - 1) * 50 + idx + 1 }}</td>
            <td
              v-for="col in QUOTE_LIST_COLUMNS"
              :key="col"
              :class="{ 'cell-copied': copiedKey === `${idx}-${col}` }"
              @dblclick="copyCell(`${idx}-${col}`, row[col] ?? '')"
            >
              <span
                v-if="col === '利润' && row[col]"
                :class="['profit-tag', profitClass(row[col])]"
              >{{ row[col] }}</span>
              <template v-else>{{ row[col] ?? '' }}</template>
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
  </div>
</template>

<style scoped>
.quote-view { display: flex; flex-direction: column; height: 100%; overflow: hidden; background: #f0f2f5; }

.toast { position: fixed; top: 20px; right: 20px; padding: 10px 18px; border-radius: 6px; font-size: 13px; z-index: 9999; box-shadow: 0 4px 16px rgba(0,0,0,0.15); }
.toast-error   { background: #ff4d4f; color: #fff; }
.toast-success { background: #52c41a; color: #fff; }
.fade-enter-active, .fade-leave-active { transition: opacity 0.3s; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
.copy-toast { position: fixed; bottom: 40px; left: 50%; transform: translateX(-50%); background: #52c41a; color: #fff; padding: 6px 20px; border-radius: 20px; font-size: 13px; z-index: 9999; pointer-events: none; }
.slide-up-enter-active, .slide-up-leave-active { transition: all 0.25s ease; }
.slide-up-enter-from, .slide-up-leave-to { opacity: 0; transform: translateX(-50%) translateY(10px); }

.top-bar { display: flex; align-items: center; justify-content: space-between; padding: 0 16px; height: 52px; background: #fff; border-bottom: 1px solid #e8e8e8; flex-shrink: 0; }
.page-title { font-size: 16px; font-weight: 600; color: #262626; }
.top-actions { display: flex; align-items: center; gap: 8px; }
.search-input { padding: 5px 10px; background: #fff; border: 1px solid #d9d9d9; border-radius: 4px; color: #333; font-size: 13px; }
.search-input:focus { outline: none; border-color: #1677ff; }
.search-input-sm { width: 110px; }
.btn-search { padding: 5px 12px; background: #1677ff; border: none; border-radius: 4px; color: #fff; font-size: 13px; cursor: pointer; transition: background 0.2s; }
.btn-search:hover { background: #4096ff; }
.btn-reset { padding: 5px 12px; background: #fff; border: 1px solid #d9d9d9; border-radius: 4px; color: #595959; font-size: 13px; cursor: pointer; transition: all 0.2s; }
.btn-reset:hover { border-color: #1677ff; color: #1677ff; }

.stat-bar { padding: 4px 16px; font-size: 12px; color: #8c8c8c; background: #fafafa; border-bottom: 1px solid #f0f0f0; flex-shrink: 0; }
.stat-bar b { color: #1677ff; }

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
.col-seq { width: 46px; text-align: center; color: #bfbfbf !important; background: #fafafa !important; }

.profit-tag { display: inline-block; padding: 1px 8px; border-radius: 10px; font-size: 12px; font-weight: 500; white-space: nowrap; }
.profit-pos { color: #52c41a; background: #f6ffed; border: 1px solid #b7eb8f; }
.profit-neg { color: #ff4d4f; background: #fff2f0; border: 1px solid #ffccc7; }

.pagination { display: flex; align-items: center; justify-content: center; gap: 12px; padding: 8px; background: #fff; border-top: 1px solid #f0f0f0; flex-shrink: 0; }
.pagination button { padding: 4px 12px; background: #fff; border: 1px solid #d9d9d9; border-radius: 4px; color: #595959; font-size: 13px; cursor: pointer; transition: all 0.2s; }
.pagination button:disabled { opacity: 0.4; cursor: not-allowed; }
.pagination button:hover:not(:disabled) { border-color: #1677ff; color: #1677ff; }
.pagination span { color: #595959; font-size: 13px; min-width: 60px; text-align: center; }
</style>
