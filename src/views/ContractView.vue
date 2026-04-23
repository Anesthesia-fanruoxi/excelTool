<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import { useContractView } from '../composables/useContractView';
import { useNavigationStore } from '../stores/navigation';

const nav = useNavigationStore();

const {
  isLoading, contracts, keyword, curPage, pageSize,
  totalPages, pagedContracts, expandedNo, expandedRows,
  toastMsg, toastType,
  onSearch, onReset, loadData, toggleExpand,
} = useContractView();

function handleJump() {
  if (!nav.jumpContractNo) return;
  keyword.value = nav.jumpContractNo;
  onSearch();
  // 等数据加载完再展开
  setTimeout(() => {
    toggleExpand(nav.jumpContractNo);
    nav.clearJump();
  }, 300);
}

onMounted(handleJump);
watch(() => nav.jumpContractNo, handleJump);

function fmt(val: number): string {
  return val.toLocaleString('zh-CN', { minimumFractionDigits: 2, maximumFractionDigits: 2 });
}
function profitClass(val: number) { return val >= 0 ? 'profit-pos' : 'profit-neg'; }
function profitRate(sale: number, profit: number): string {
  if (sale <= 0) return '—';
  return (profit / sale * 100).toFixed(1) + '%';
}

// ── 行内编辑 ──────────────────────────────────────────────
// editingId → { quantity, unit_price }
const editing = ref<Record<number, { quantity: string; unit_price: string }>>({});
const savingId = ref<number | null>(null);

function startEdit(row: any) {
  editing.value[row.id] = {
    quantity:   String(row.quantity),
    unit_price: String(row.unit_price),
  };
}

function cancelEdit(id: number) {
  delete editing.value[id];
}

async function saveEdit(row: any, contractNo: string) {
  const e = editing.value[row.id];
  if (!e) return;
  const qty = parseFloat(e.quantity);
  const price = parseFloat(e.unit_price);
  if (isNaN(qty) || qty < 0)   { alert('数量无效'); return; }
  if (isNaN(price) || price < 0) { alert('单价无效'); return; }

  savingId.value = row.id;
  try {
    await invoke('update_sales_item_price', { id: row.id, quantity: qty, unitPrice: price });
    // 更新本地数据
    row.quantity   = qty;
    row.unit_price = price;
    row.sale_amount = Math.round(qty * price * 100) / 100;
    delete editing.value[row.id];
    // 刷新合同汇总
    loadData();
  } catch (err) {
    alert(`保存失败: ${err}`);
  } finally {
    savingId.value = null;
  }
}
</script>

<template>
  <div class="view-wrap">
    <transition name="fade">
      <div v-if="toastMsg" :class="['toast', toastType === 'error' ? 'toast-error' : 'toast-success']">{{ toastMsg }}</div>
    </transition>

    <div class="top-bar">
      <span class="page-title">合同管理</span>
      <div class="top-actions">
        <input v-model="keyword" class="search-input" placeholder="合同号 / 客户 / 项目名称" @keydown.enter="onSearch" />
        <button class="btn-search" @click="onSearch">搜索</button>
        <button class="btn-reset" @click="onReset">重置</button>
      </div>
    </div>

    <div class="stat-bar">共 <b>{{ contracts.length }}</b> 份合同</div>

    <div class="table-wrap">
      <div v-if="isLoading" class="status-center">加载中...</div>
      <div v-else-if="contracts.length === 0" class="status-center">
        <div class="empty-icon">📄</div>
        <p>暂无合同数据，请先导入销售表</p>
      </div>
      <table v-else class="data-table">
        <thead>
          <tr>
            <th class="col-seq">#</th>
            <th>合同号</th>
            <th>客户</th>
            <th>销售日期</th>
            <th>项目名称</th>
            <th class="num-col">产品数</th>
            <th class="num-col">销售金额</th>
            <th class="num-col">成本金额</th>
            <th class="num-col">利润</th>
            <th class="num-col">利润率</th>
            <th class="col-warn">未关联</th>
          </tr>
        </thead>
        <tbody>
          <template v-for="(c, idx) in pagedContracts" :key="c.contract_no">
            <tr class="contract-row" @click="toggleExpand(c.contract_no)">
              <td class="col-seq">{{ (curPage - 1) * pageSize + idx + 1 }}</td>
              <td class="contract-no">{{ c.contract_no }}</td>
              <td>{{ c.customer }}</td>
              <td>{{ c.sale_date }}</td>
              <td>{{ c.project_name }}</td>
              <td class="num-col"><span class="badge">{{ c.product_count }}</span></td>
              <td class="num-col amount">{{ fmt(c.total_sale_amount) }}</td>
              <td class="num-col">{{ c.total_cost_amount > 0 ? fmt(c.total_cost_amount) : '—' }}</td>
              <td class="num-col" :class="profitClass(c.total_profit)">
                {{ c.total_cost_amount > 0 ? fmt(c.total_profit) : '—' }}
              </td>
              <td class="num-col" :class="profitClass(c.total_profit)">
                {{ c.total_cost_amount > 0 ? profitRate(c.total_sale_amount, c.total_profit) : '—' }}
              </td>
              <td class="col-warn">
                <span v-if="c.unlinked_count > 0" class="tag-warn">{{ c.unlinked_count }} 条</span>
                <span v-else class="tag-ok">✓</span>
              </td>
            </tr>

            <!-- 展开明细 -->
            <tr v-if="expandedNo === c.contract_no" class="detail-row">
              <td colspan="11" class="detail-cell">
                <table class="detail-table">
                  <thead>
                    <tr>
                      <th>产品名称</th>
                      <th>规格</th>
                      <th>数量</th>
                      <th>单位</th>
                      <th>销售单价</th>
                      <th>销售金额</th>
                      <th>报价物品</th>
                      <th>成本单价</th>
                      <th>成本金额</th>
                      <th>利润</th>
                      <th>税率</th>
                      <th>供应商</th>
                      <th>备注</th>
                      <th class="col-row-op"></th>
                    </tr>
                  </thead>
                  <tbody>
                    <tr
                      v-for="row in expandedRows[c.contract_no] || []"
                      :key="row.id"
                      :class="{ 'row-unlinked': !row.item_uuid }"
                    >
                      <td>{{ row.product_name }}</td>
                      <td>{{ row.spec }}</td>
                      <td class="num-col">
                        <template v-if="editing[row.id]">
                          <input v-model="editing[row.id].quantity" class="edit-inp" type="number" min="0" />
                        </template>
                        <template v-else>{{ row.quantity }}</template>
                      </td>
                      <td>{{ row.unit }}</td>
                      <td class="num-col">
                        <template v-if="editing[row.id]">
                          <input v-model="editing[row.id].unit_price" class="edit-inp" type="number" min="0" step="0.01" />
                        </template>
                        <template v-else>{{ row.unit_price.toFixed(2) }}</template>
                      </td>
                      <td class="num-col">{{ fmt(row.sale_amount) }}</td>
                      <td>
                        <span v-if="row.goods_name" class="goods-name">{{ row.goods_name }}</span>
                        <span v-else class="tag-unlinked">未关联</span>
                      </td>
                      <td class="num-col">{{ row.cost_price != null ? row.cost_price.toFixed(2) : '—' }}</td>
                      <td class="num-col">{{ row.cost_amount != null ? fmt(row.cost_amount) : '—' }}</td>
                      <td class="num-col" :class="row.profit != null ? profitClass(row.profit) : ''">
                        {{ row.profit != null ? fmt(row.profit) : '—' }}
                      </td>
                      <td>{{ row.tax_rate ?? '—' }}</td>
                      <td>{{ row.supplier }}</td>
                      <td>{{ row.remark }}</td>
                      <td class="col-row-op">
                        <template v-if="editing[row.id]">
                          <button class="btn-row-save" :disabled="savingId === row.id" @click="saveEdit(row, c.contract_no)">
                            {{ savingId === row.id ? '...' : '保存' }}
                          </button>
                          <button class="btn-row-cancel" @click="cancelEdit(row.id)">取消</button>
                        </template>
                        <template v-else>
                          <button class="btn-row-edit" @click="startEdit(row)">编辑</button>
                        </template>
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

    <div v-if="contracts.length > 0" class="pagination">
      <button :disabled="curPage === 1" @click="curPage--">上一页</button>
      <span>{{ curPage }} / {{ totalPages }}</span>
      <button :disabled="curPage >= totalPages" @click="curPage++">下一页</button>
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
.top-bar { display: flex; align-items: center; justify-content: space-between; padding: 0 16px; height: 52px; background: #fff; border-bottom: 1px solid #e8e8e8; flex-shrink: 0; }
.page-title { font-size: 16px; font-weight: 600; color: #262626; }
.top-actions { display: flex; align-items: center; gap: 8px; }
.search-input { padding: 5px 10px; background: #fff; border: 1px solid #d9d9d9; border-radius: 4px; color: #333; font-size: 13px; width: 240px; }
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
.contract-row { cursor: pointer; }
.contract-row td { background: #fff; color: #333; }
.contract-row:hover td { background: #e6f4ff; }
.col-seq { width: 46px; text-align: center; color: #bfbfbf !important; background: #fafafa !important; }
.contract-no { color: #1677ff !important; font-weight: 500; }
.num-col { text-align: right; font-variant-numeric: tabular-nums; }
.amount { color: #262626 !important; font-weight: 500; }
.profit-pos { color: #52c41a !important; }
.profit-neg { color: #ff4d4f !important; }
.badge { display: inline-block; padding: 1px 8px; background: #f0f0f0; border-radius: 10px; font-size: 12px; color: #8c8c8c; }
.col-warn { width: 80px; text-align: center; }
.tag-warn { display: inline-block; padding: 1px 8px; border-radius: 10px; font-size: 11px; background: #fff7e6; color: #fa8c16; border: 1px solid #ffd591; }
.tag-ok { color: #52c41a; font-size: 13px; }
.tag-unlinked { display: inline-block; padding: 1px 6px; border-radius: 10px; font-size: 11px; background: #fff2f0; color: #ff4d4f; border: 1px solid #ffccc7; }
/* 明细行 */
.detail-row td { padding: 0; background: #fafafa !important; }
.detail-cell { padding: 0 !important; }
.detail-table { width: 100%; border-collapse: collapse; font-size: 12px; }
.detail-table th, .detail-table td { padding: 5px 8px; border-bottom: 1px solid #f0f0f0; border-right: 1px solid #f0f0f0; text-align: left; white-space: nowrap; }
.detail-table th:last-child, .detail-table td:last-child { border-right: none; }
.detail-table thead th { background: #f5f5f5; color: #8c8c8c; font-weight: 500; }
.detail-table tbody td { color: #595959; background: #fafafa; }
.row-unlinked td { background: #fffbe6 !important; }
.goods-name { font-size: 11px; color: #1677ff; }
.col-row-op { width: 100px; text-align: center; white-space: nowrap; }
.edit-inp { width: 70px; padding: 2px 4px; border: 1px solid #1677ff; border-radius: 3px; font-size: 12px; text-align: right; }
.edit-inp:focus { outline: none; }
.btn-row-edit { padding: 1px 8px; background: #fff; border: 1px solid #d9d9d9; border-radius: 3px; color: #1677ff; font-size: 11px; cursor: pointer; }
.btn-row-edit:hover { background: #1677ff; color: #fff; border-color: #1677ff; }
.btn-row-save { padding: 1px 8px; background: #1677ff; border: none; border-radius: 3px; color: #fff; font-size: 11px; cursor: pointer; }
.btn-row-save:disabled { opacity: 0.5; cursor: not-allowed; }
.btn-row-cancel { padding: 1px 8px; background: #fff; border: 1px solid #d9d9d9; border-radius: 3px; color: #595959; font-size: 11px; cursor: pointer; margin-left: 3px; }
.btn-row-cancel:hover { border-color: #ff4d4f; color: #ff4d4f; }
.pagination { display: flex; align-items: center; justify-content: center; gap: 12px; padding: 8px; background: #fff; border-top: 1px solid #f0f0f0; flex-shrink: 0; }
.pagination button { padding: 4px 12px; background: #fff; border: 1px solid #d9d9d9; border-radius: 4px; color: #595959; font-size: 13px; cursor: pointer; }
.pagination button:disabled { opacity: 0.4; cursor: not-allowed; }
.pagination button:hover:not(:disabled) { border-color: #1677ff; color: #1677ff; }
.pagination span { color: #595959; font-size: 13px; min-width: 60px; text-align: center; }
</style>
