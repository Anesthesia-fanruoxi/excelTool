<script setup lang="ts">
import { useContractView } from '../composables/useContractView';
import { useCopyCell } from '../composables/useCopyCell';

const {
  isLoading,
  searchText,
  statusFilter,
  curPage,
  pageSize,
  totalPages,
  pagedContracts,
  filteredContracts,
  expandedContract,
  expandedRows,
  toastMsg,
  toastType,
  onSearch,
  toggleExpand,
} = useContractView();

const { copiedKey, toastVisible, copyCell } = useCopyCell();

function fmt(val: number): string {
  return val.toLocaleString('zh-CN', { minimumFractionDigits: 2, maximumFractionDigits: 2 });
}
</script>

<template>
  <div class="contract-view">
    <!-- Toast -->
    <transition name="fade">
      <div v-if="toastMsg" :class="['toast', toastType === 'error' ? 'toast-error' : 'toast-success']">
        {{ toastMsg }}
      </div>
    </transition>

    <!-- 复制提示 -->
    <transition name="slide-up">
      <div v-if="toastVisible" class="copy-toast">复制成功 ✓</div>
    </transition>

    <!-- 顶部栏 -->
    <div class="top-bar">
      <span class="page-title">合同</span>
      <div class="top-actions">
        <select v-model="statusFilter" class="sel" @change="onSearch">
          <option value="">全部状态</option>
          <option value="已对账">已对账</option>
          <option value="待对账">待对账</option>
          <option value="等回签">等回签</option>
          <option value="回签不完整">回签不完整</option>
        </select>
        <input v-model="searchText" class="search-input" placeholder="搜索合同号 / 客户 / 项目..." @input="onSearch" />
      </div>
    </div>

    <!-- 统计栏 -->
    <div class="stat-bar">
      共 <b>{{ filteredContracts.length }}</b> 份合同
    </div>

    <!-- 表格 -->
    <div class="table-wrap">
      <div v-if="isLoading" class="status-center">加载中...</div>
      <div v-else-if="filteredContracts.length === 0" class="status-center">
        <div class="empty-icon">📄</div>
        <p>暂无合同数据，请先在「数据管理」中导入</p>
      </div>

      <table v-else class="data-table">
        <thead>
          <tr>
            <th class="col-expand"></th>
            <th class="col-seq">#</th>
            <th>合同号</th>
            <th>客户</th>
            <th>销售日期</th>
            <th>项目名称</th>
            <th class="num-col">明细数</th>
            <th class="num-col">金额合计</th>
            <th class="num-col">利润合计</th>
            <th class="col-status">对账状态</th>
          </tr>
        </thead>
        <tbody>
          <template v-for="(contract, idx) in pagedContracts" :key="contract.contract_no">
            <!-- 合同主行 -->
            <tr class="contract-row" @click="toggleExpand(contract.contract_no)">
              <td class="col-expand">
                <span class="expand-icon">{{ expandedContract === contract.contract_no ? '▼' : '▶' }}</span>
              </td>
              <td class="col-seq">{{ (curPage - 1) * pageSize + idx + 1 }}</td>
              <td
                class="contract-no"
                :class="{ 'cell-copied': copiedKey === `${idx}-合同号` }"
                @dblclick.stop="copyCell(`${idx}-合同号`, contract.contract_no)"
              >{{ contract.contract_no }}</td>
              <td
                :class="{ 'cell-copied': copiedKey === `${idx}-客户` }"
                @dblclick.stop="copyCell(`${idx}-客户`, contract.customer)"
              >{{ contract.customer }}</td>
              <td>{{ contract.sale_date }}</td>
              <td
                :class="{ 'cell-copied': copiedKey === `${idx}-项目` }"
                @dblclick.stop="copyCell(`${idx}-项目`, contract.project_name)"
              >{{ contract.project_name }}</td>
              <td class="num-col">
                <span class="badge">{{ contract.row_count }}</span>
              </td>
              <td class="num-col amount">{{ fmt(contract.total_amount) }}</td>
              <td class="num-col" :class="contract.total_profit >= 0 ? 'profit-pos' : 'profit-neg'">
                {{ fmt(contract.total_profit) }}
              </td>
              <td class="col-status">
                <span :class="['status-tag', `status-${contract.reconcile_status}`]">
                  {{ contract.reconcile_status }}
                </span>
              </td>
            </tr>

            <!-- 展开明细行 -->
            <tr v-if="expandedContract === contract.contract_no" class="detail-row">
              <td colspan="9" class="detail-cell">
                <table class="detail-table">
                  <thead>
                    <tr>
                      <th>序号</th>
                      <th>产品名称</th>
                      <th>规格</th>
                      <th>数量</th>
                      <th>单位</th>
                      <th>单价</th>
                      <th>金额</th>
                      <th>安装位置</th>
                      <th>利润</th>
                      <th>对账状态</th>
                    </tr>
                  </thead>
                  <tbody>
                    <tr v-for="(row, ri) in expandedRows[contract.contract_no] || []" :key="ri">
                      <td>{{ row['序号'] }}</td>
                      <td>{{ row['产品名称'] }}</td>
                      <td>{{ row['规格'] }}</td>
                      <td>{{ row['数量'] }}</td>
                      <td>{{ row['单位'] }}</td>
                      <td>{{ row['单价'] }}</td>
                      <td>{{ row['金额'] }}</td>
                      <td>{{ row['安装位置'] }}</td>
                      <td :class="parseFloat(row['利润']) >= 0 ? 'profit-pos' : 'profit-neg'">
                        {{ row['利润'] }}
                      </td>
                      <td>
                        <span :class="['status-tag', `status-${row['状态列']}`]">
                          {{ row['状态列'] || '—' }}
                        </span>
                      </td>
                    </tr>
                  </tbody>
                </table>
              </td>
            </tr>
          </template>
        </tbody>
      </table>
    </div>

    <!-- 分页 -->
    <div v-if="filteredContracts.length > 0" class="pagination">
      <button :disabled="curPage === 1" @click="curPage--">上一页</button>
      <span>{{ curPage }} / {{ totalPages }}</span>
      <button :disabled="curPage >= totalPages" @click="curPage++">下一页</button>
    </div>
  </div>
</template>

<style scoped>
.contract-view {
  display: flex; flex-direction: column;
  height: 100%; overflow: hidden;
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
  position: fixed; bottom: 40px; left: 50%; transform: translateX(-50%);
  background: #27ae60; color: #fff; padding: 8px 24px;
  border-radius: 20px; font-size: 13px; z-index: 9999;
  box-shadow: 0 4px 12px rgba(0,0,0,0.3); pointer-events: none;
}
.slide-up-enter-active, .slide-up-leave-active { transition: all 0.25s ease; }
.slide-up-enter-from, .slide-up-leave-to { opacity: 0; transform: translateX(-50%) translateY(10px); }

.top-bar {
  display: flex; align-items: center; justify-content: space-between;
  padding: 0 20px; height: 56px; border-bottom: 1px solid #0f3460; flex-shrink: 0;
}
.page-title { font-size: 17px; font-weight: 600; color: #fff; }
.top-actions { display: flex; align-items: center; gap: 8px; }
.search-input {
  padding: 6px 12px; background: #0f3460; border: 1px solid #1a4a7a;
  border-radius: 4px; color: #fff; font-size: 13px; width: 280px;
}
.search-input:focus { outline: none; border-color: #4a9eff; }
.sel {
  padding: 6px 10px; background: #0f3460; border: 1px solid #1a4a7a;
  border-radius: 4px; color: #ccc; font-size: 13px; cursor: pointer;
}
.sel:focus { outline: none; border-color: #4a9eff; }

.stat-bar {
  padding: 6px 20px; font-size: 12px; color: #666;
  border-bottom: 1px solid #0f3460; flex-shrink: 0;
}
.stat-bar b { color: #4a9eff; }

.table-wrap { flex: 1; overflow: auto; }

.status-center {
  display: flex; flex-direction: column; align-items: center;
  justify-content: center; height: 100%; color: #555; gap: 12px; font-size: 14px;
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

.contract-row { cursor: pointer; }
.contract-row td { background: #1a1a2e; color: #ccc; }
.contract-row:hover td { background: #16213e; }

.col-expand { width: 36px; text-align: center; }
.expand-icon { font-size: 10px; color: #4a9eff; }
.col-seq {
  width: 50px; text-align: center;
  background: #0f3460 !important; color: #555 !important;
}
.contract-no { color: #4a9eff !important; font-weight: 500; }
.num-col { text-align: right; font-variant-numeric: tabular-nums; }
.amount { color: #fff !important; }
.profit-pos { color: #27ae60 !important; }
.profit-neg { color: #e74c3c !important; }
.badge {
  display: inline-block; padding: 1px 8px;
  background: #0f3460; border-radius: 10px;
  font-size: 12px; color: #888;
}
.cell-copied { color: #27ae60 !important; background: #0d2a1a !important; }

.col-status { width: 100px; text-align: center; }
.status-tag {
  display: inline-block; padding: 2px 10px; border-radius: 10px;
  font-size: 12px; font-weight: 500; white-space: nowrap;
}
.status-已对账  { background: #0d2a1a; color: #27ae60; border: 1px solid #27ae6044; }
.status-待对账  { background: #1a2a0a; color: #f39c12; border: 1px solid #f39c1244; }
.status-等回签  { background: #0a1a2a; color: #4a9eff; border: 1px solid #4a9eff44; }
.status-回签不完整 { background: #2a1010; color: #e74c3c; border: 1px solid #e74c3c44; }

/* 展开明细 */
.detail-row td { padding: 0; background: #111827 !important; }
.detail-cell { padding: 0 !important; }
.detail-table {
  width: 100%; border-collapse: collapse; font-size: 12px;
}
.detail-table th,
.detail-table td {
  padding: 7px 12px; border: 1px solid #0f3460;
  text-align: left; white-space: nowrap;
}
.detail-table thead th {
  background: #0f1f35; color: #888; font-weight: 500;
  position: sticky; top: 0;
}
.detail-table tbody td { color: #aaa; background: #111827; }

.pagination {
  display: flex; align-items: center; justify-content: center;
  gap: 14px; padding: 10px; border-top: 1px solid #0f3460; flex-shrink: 0;
}
.pagination button {
  padding: 5px 14px; background: #0f3460; border: none;
  border-radius: 4px; color: #ccc; font-size: 13px; cursor: pointer; transition: all 0.2s;
}
.pagination button:disabled { opacity: 0.4; cursor: not-allowed; }
.pagination button:hover:not(:disabled) { background: #4a9eff; color: #fff; }
.pagination span { color: #ccc; font-size: 13px; min-width: 60px; text-align: center; }
</style>
