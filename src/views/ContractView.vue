<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import * as XLSX from 'xlsx';
import { useContractView } from '../composables/useContractView';
import { useCopyCell } from '../composables/useCopyCell';
import ContractAddModal from '../components/contract/ContractAddModal.vue';
import ContractEditModal from '../components/contract/ContractEditModal.vue';
import { SALES_COLUMNS } from '../constants/salesColumns';
import type { ContractSummary } from '../composables/useContractView';
import { useNavigationStore } from '../stores/navigation';

const {
  isLoading, filterContractNo, filterSaleDate, filterProjectName, filterCustomer,
  statusFilter, curPage, pageSize,
  totalPages, pagedContracts, filteredContracts,
  expandedContract, expandedRows, toastMsg, toastType,
  onSearch, onReset, loadData, toggleExpand,
} = useContractView();

const { copiedKey: _copiedKey, toastVisible, copyCell: _cc } = useCopyCell();

const showAddModal = ref(false);
const editingContract = ref<ContractSummary | null>(null);

const nav = useNavigationStore();

// 从销售明细跳转过来时，自动填入合同号并搜索展开
onMounted(() => {
  if (nav.jumpContractNo) {
    filterContractNo.value = nav.jumpContractNo;
    onSearch();
    toggleExpand(nav.jumpContractNo);
    nav.clearJump();
  }
});

function onSaved() { loadData(); }

function fmt(val: number): string {
  return val.toLocaleString('zh-CN', { minimumFractionDigits: 2, maximumFractionDigits: 2 });
}

// ── 对账状态浮层 ──────────────────────────────────────────
interface PopupState {
  row: Record<string, string>;
  form: Record<string, string>;
  x: number;
  y: number;
  signSource: Record<string, string> | null;
}
const popup = ref<PopupState | null>(null);
const isSavingPopup = ref(false);
const popupToast = ref('');

const SIGN_FIELDS = [
  { key: '签收人',        label: '签收人' },
  { key: '签收日期',      label: '签收日期' },
  { key: '与客户对账时间', label: '与客户对账时间' },
];
const RECONCILE_FIELDS = [
  { key: '对账数量', label: '对账数量' },
  { key: '对账单价', label: '对账单价' },
  { key: '对账日期', label: '对账日期' },
  { key: '对账金额', label: '对账金额' },
  { key: '对账备注', label: '对账备注' },
];

function onStatusCellClick(e: MouseEvent, row: Record<string, string>) {
  e.stopPropagation();
  const popupW = 420;
  const popupH = 380;
  const winW = window.innerWidth;
  const winH = window.innerHeight;
  const x = e.clientX + popupW > winW ? e.clientX - popupW : e.clientX;
  const y = e.clientY + popupH > winH ? e.clientY - popupH : e.clientY + 10;

  // 查找同合同其他行中已有签收信息的行
  const contractNo = row['合同号'];
  const siblings = expandedRows.value[contractNo] ?? [];
  const signSource = siblings.find(r =>
    r['__id'] !== row['__id'] && (r['签收人'] || r['签收日期'])
  );

  popup.value = {
    row,
    form: { ...row },
    x: Math.max(0, x),
    y: Math.max(0, y),
    signSource: signSource ?? null,
  };
  popupToast.value = '';
  window.addEventListener('keydown', onPopupKeydown);
}

function applySignSource() {
  if (!popup.value?.signSource) return;
  const src = popup.value.signSource;
  popup.value.form['签收人'] = src['签收人'] ?? '';
  popup.value.form['签收日期'] = src['签收日期'] ?? '';
  popup.value.form['与客户对账时间'] = src['与客户对账时间'] ?? '';
}

function closePopup() {
  popup.value = null;
  window.removeEventListener('keydown', onPopupKeydown);
}

function onPopupKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') closePopup();
}

async function savePopup() {
  if (!popup.value) return;
  isSavingPopup.value = true;
  try {
    const id = Number(popup.value.form['__id']);
    const rowData = { ...popup.value.row, ...popup.value.form };
    delete rowData['__id'];
    await invoke('save_sales_row', { id, rowData });
    Object.assign(popup.value.row, popup.value.form);
    popupToast.value = '保存成功';
    loadData();
    if (expandedContract.value) {
      expandedRows.value[expandedContract.value] = await invoke('query_contract_detail', {
        contractNo: expandedContract.value,
      });
    }
    setTimeout(() => closePopup(), 800);
  } catch (e) {
    popupToast.value = `保存失败: ${e}`;
  } finally {
    isSavingPopup.value = false;
  }
}

// ── 导出合同明细（全列）────────────────────────────────────
async function exportContract(e: MouseEvent, contractNo: string) {
  e.stopPropagation();
  try {
    const rows: Record<string, string>[] = await invoke('query_contract_detail', { contractNo });
    const headers = [...SALES_COLUMNS];
    const data = [
      headers,
      ...rows.map(r => headers.map(h => {
        if (h === '税率' && r[h]) return `${Math.round(parseFloat(r[h]) * 100)}%`;
        return r[h] ?? '';
      })),
    ];
    const ws = XLSX.utils.aoa_to_sheet(data);
    ws['!cols'] = headers.map(h => ({ wch: Math.max(h.length * 2, 10) }));
    const wb = XLSX.utils.book_new();
    XLSX.utils.book_append_sheet(wb, ws, '明细');
    XLSX.writeFile(wb, `合同_${contractNo}.xlsx`);
  } catch (err) {
    console.error('导出失败', err);
  }
}
</script>

<template>
  <div class="contract-view">
    <!-- Toast -->
    <transition name="fade">
      <div v-if="toastMsg" :class="['toast', toastType === 'error' ? 'toast-error' : 'toast-success']">{{ toastMsg }}</div>
    </transition>
    <transition name="slide-up">
      <div v-if="toastVisible" class="copy-toast">复制成功 ✓</div>
    </transition>

    <!-- 顶部栏 -->
    <div class="top-bar">
      <span class="page-title">合同管理</span>
      <div class="top-actions">
        <select v-model="statusFilter" class="sel" @change="onSearch">
          <option value="">全部状态</option>
          <option value="已对账">已对账</option>
          <option value="待对账">待对账</option>
          <option value="等回签">等回签</option>
          <option value="回签不完整">回签不完整</option>
        </select>
        <input
          v-model="filterContractNo"
          class="search-input search-input-sm"
          placeholder="合同号"
          @keydown.enter="onSearch"
        />
        <input
          v-model="filterSaleDate"
          class="search-input search-input-sm"
          placeholder="销售日期"
          @keydown.enter="onSearch"
        />
        <input
          v-model="filterProjectName"
          class="search-input search-input-sm"
          placeholder="项目名称"
          @keydown.enter="onSearch"
        />
        <input
          v-model="filterCustomer"
          class="search-input search-input-sm"
          placeholder="客户名称"
          @keydown.enter="onSearch"
        />
        <button class="btn-search" @click="onSearch">搜索</button>
        <button class="btn-reset" @click="onReset">重置</button>
        <button class="btn-add" @click="showAddModal = true">+ 新增合同</button>
      </div>
    </div>

    <!-- 统计栏 -->
    <div class="stat-bar">共 <b>{{ filteredContracts.length }}</b> 份合同</div>

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
            <th class="col-seq">#</th>
            <th>合同号</th>
            <th>客户</th>
            <th>销售日期</th>
            <th>项目名称</th>
            <th class="num-col">明细数</th>
            <th class="num-col">金额合计</th>
            <th class="num-col">利润合计</th>
            <th class="col-status">对账状态</th>
            <th class="col-contract-op">操作</th>
          </tr>
        </thead>
        <tbody>
          <template v-for="(contract, idx) in pagedContracts" :key="contract.contract_no">
            <tr class="contract-row" @click="toggleExpand(contract.contract_no)">
              <td class="col-seq">{{ (curPage - 1) * pageSize + idx + 1 }}</td>
              <td class="contract-no">{{ contract.contract_no }}</td>
              <td>{{ contract.customer }}</td>
              <td>{{ contract.sale_date }}</td>
              <td>{{ contract.project_name }}</td>
              <td class="num-col"><span class="badge">{{ contract.row_count }}</span></td>
              <td class="num-col amount">{{ fmt(contract.total_amount) }}</td>
              <td class="num-col" :class="contract.total_profit >= 0 ? 'profit-pos' : 'profit-neg'">{{ fmt(contract.total_profit) }}</td>
              <td class="col-status">
                <span :class="['status-tag', `status-${contract.reconcile_status}`]">{{ contract.reconcile_status }}</span>
              </td>
              <td class="col-contract-op" @click.stop>
                <button class="btn-contract-edit" @click="editingContract = contract">编辑</button>
                <button class="btn-contract-export" @click="exportContract($event, contract.contract_no)">导出</button>
              </td>
            </tr>
            <tr v-if="expandedContract === contract.contract_no" class="detail-row">
              <td colspan="12" class="detail-cell">
                <table class="detail-table">
                  <thead>
                    <tr>
                      <th>序号</th><th>产品名称</th><th>规格</th><th>数量</th>
                      <th>单位</th><th>单价</th><th>金额</th><th>安装位置</th>
                      <th>初始报价</th><th>税率</th><th>成本单价含税</th>
                      <th>利润</th><th>对账状态</th>
                    </tr>
                  </thead>
                  <tbody>
                    <tr
                      v-for="(row, ri) in expandedRows[contract.contract_no] || []"
                      :key="ri"
                      class="detail-data-row"
                    >
                      <td>{{ row['序号'] }}</td>
                      <td>{{ row['产品名称'] }}</td>
                      <td>{{ row['规格'] }}</td>
                      <td>{{ row['数量'] }}</td>
                      <td>{{ row['单位'] }}</td>
                      <td>{{ row['单价'] }}</td>
                      <td>{{ row['金额'] }}</td>
                      <td>{{ row['安装位置'] }}</td>
                      <td>{{ row['初始报价'] }}</td>
                      <td>{{ row['税率'] ? `${Math.round(parseFloat(row['税率']) * 100)}%` : '' }}</td>
                      <td>{{ row['成本单价含税'] }}</td>
                      <td :class="parseFloat(row['利润']) >= 0 ? 'profit-pos' : 'profit-neg'">{{ row['利润'] }}</td>
                      <td class="col-status-cell" @click.stop="onStatusCellClick($event, row)">
                        <span :class="['status-tag', `status-${row['状态列']}`]">{{ row['状态列'] || '—' }}</span>
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

    <!-- 明细行浮层 -->
    <teleport to="body">
      <div v-if="popup" class="row-popup-mask">
        <div
          class="row-popup"
          :style="{ left: `${popup.x}px`, top: `${popup.y}px` }"
          @click.stop
        >
          <!-- 标题 -->
          <div class="popup-title">
            <span :class="['status-tag', `status-${popup.row['状态列']}`]">
              {{ popup.row['状态列'] || '无状态' }}
            </span>
            {{ popup.row['状态列'] === '已对账' ? '对账信息' : '签收 / 对账进度' }}
            <button class="popup-close" @click="closePopup">×</button>
          </div>

          <!-- 编辑表单：签收信息 + 对账信息 -->
          <div class="popup-grid">
            <div class="popup-section-label">
              签收信息
              <button
                v-if="popup.signSource && (!popup.form['签收人'] && !popup.form['签收日期'])"
                class="btn-apply-sign"
                @click="applySignSource"
              >
                ↙ 填入已有签收信息（{{ popup.signSource['签收人'] }} {{ popup.signSource['签收日期'] }}）
              </button>
            </div>
            <div v-for="f in SIGN_FIELDS" :key="f.key"
              :class="['popup-item', f.key === '与客户对账时间' ? 'popup-item-full' : '']">
              <label class="pk">{{ f.label }}</label>
              <input v-model="popup.form[f.key]" class="popup-input" :placeholder="f.label" />
            </div>
            <div class="popup-section-label popup-section-label-2">对账信息</div>
            <div v-for="f in RECONCILE_FIELDS" :key="f.key"
              :class="['popup-item', f.key === '对账备注' ? 'popup-item-full' : '']">
              <label class="pk">{{ f.label }}</label>
              <input v-model="popup.form[f.key]" class="popup-input" :placeholder="f.label" />
            </div>
          </div>

          <!-- 底部 -->
          <div class="popup-foot">
            <span v-if="popupToast" :class="['popup-toast', popupToast.includes('失败') ? 'pt-error' : 'pt-success']">
              {{ popupToast }}
            </span>
            <button class="popup-btn-cancel" @click="closePopup">取消</button>
            <button class="popup-btn-save" :disabled="isSavingPopup" @click="savePopup">
              {{ isSavingPopup ? '保存中...' : '保存' }}
            </button>
          </div>
        </div>
      </div>
    </teleport>

    <!-- 新增合同弹框 -->
    <ContractAddModal v-if="showAddModal" @close="showAddModal = false" @saved="onSaved" />

    <!-- 编辑合同弹框 -->
    <ContractEditModal
      v-if="editingContract"
      :contract="editingContract"
      @close="editingContract = null"
      @saved="onSaved"
    />
  </div>
</template>

<style scoped>
.contract-view { display: flex; flex-direction: column; height: 100%; overflow: hidden; background: #f0f2f5; }

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
.search-input { padding: 5px 10px; background: #fff; border: 1px solid #d9d9d9; border-radius: 4px; color: #333; font-size: 13px; width: 260px; }
.search-input:focus { outline: none; border-color: #1677ff; }
.search-input-sm { width: 110px; }
.btn-add { padding: 5px 14px; background: #1677ff; border: none; border-radius: 4px; color: #fff; font-size: 13px; cursor: pointer; transition: background 0.2s; }
.btn-add:hover { background: #4096ff; }
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

.contract-row { cursor: pointer; }
.contract-row td { background: #fff; color: #333; }
.contract-row:hover td { background: #e6f4ff; }

.col-expand { width: 32px; text-align: center; }
.expand-icon { font-size: 10px; color: #1677ff; }
.col-seq { width: 46px; text-align: center; color: #bfbfbf !important; background: #fafafa !important; }
.contract-no { color: #1677ff !important; font-weight: 500; }
.num-col { text-align: right; font-variant-numeric: tabular-nums; }
.amount { color: #262626 !important; font-weight: 500; }
.profit-pos { color: #52c41a !important; }
.profit-neg { color: #ff4d4f !important; }
.badge { display: inline-block; padding: 1px 8px; background: #f0f0f0; border-radius: 10px; font-size: 12px; color: #8c8c8c; }
.cell-copied { color: #52c41a !important; background: #f6ffed !important; }

.col-status { width: 100px; text-align: center; }
.col-contract-op { width: 130px; text-align: center; white-space: nowrap; }
.btn-contract-edit {
  padding: 2px 8px; background: #1677ff; border: none;
  border-radius: 3px; color: #fff; font-size: 11px; cursor: pointer; transition: all 0.15s;
}
.btn-contract-edit:hover { background: #4096ff; }
.col-status-cell { cursor: pointer; text-align: center; }
.col-status-cell:hover { background: #f0f5ff !important; }
.status-tag { display: inline-block; padding: 1px 8px; border-radius: 10px; font-size: 12px; font-weight: 500; white-space: nowrap; }
.status-已对账    { background: #f6ffed; color: #52c41a; border: 1px solid #b7eb8f; }
.status-待对账    { background: #f9f0ff; color: #722ed1; border: 1px solid #d3adf7; }
.status-等回签    { background: #fff2f0; color: #ff4d4f; border: 1px solid #ffccc7; }
.status-回签不完整 { background: #e6f4ff; color: #1677ff; border: 1px solid #91caff; }

.detail-data-row { cursor: pointer; }
.detail-data-row:hover td { background: #e6f4ff !important; }

.col-op-th { width: 80px; text-align: center; }
.col-op-td { text-align: center; white-space: nowrap; }
.btn-row-edit {
  padding: 1px 8px; background: #fff; border: 1px solid #d9d9d9;
  border-radius: 3px; color: #1677ff; font-size: 11px; cursor: pointer; transition: all 0.15s;
}
.btn-row-edit:hover { background: #1677ff; color: #fff; border-color: #1677ff; }
.btn-row-del {
  padding: 1px 8px; background: #fff; border: 1px solid #d9d9d9;
  border-radius: 3px; color: #ff4d4f; font-size: 11px; cursor: pointer; transition: all 0.15s;
}
.btn-row-del:hover { background: #ff4d4f; color: #fff; border-color: #ff4d4f; }
.btn-contract-expand {
  padding: 2px 8px; background: #fff; border: 1px solid #d9d9d9;
  border-radius: 3px; color: #595959; font-size: 11px; cursor: pointer; transition: all 0.15s;
}
.btn-contract-expand:hover { border-color: #1677ff; color: #1677ff; }
.btn-contract-export {
  padding: 2px 8px; background: #fff; border: 1px solid #d9d9d9;
  border-radius: 3px; color: #52c41a; font-size: 11px; cursor: pointer; transition: all 0.15s; margin-left: 4px;
}
.btn-contract-export:hover { background: #52c41a; color: #fff; border-color: #52c41a; }
.btn-row-save {
  padding: 1px 8px; background: #1677ff; border: none;
  border-radius: 3px; color: #fff; font-size: 11px; cursor: pointer;
}
.btn-row-save:disabled { opacity: 0.5; cursor: not-allowed; }
.btn-row-cancel {
  padding: 1px 8px; background: #fff; border: 1px solid #d9d9d9;
  border-radius: 3px; color: #595959; font-size: 11px; cursor: pointer; margin-left: 4px;
}
.edit-cell-input {
  width: 100%; min-width: 50px; padding: 2px 4px; border: 1px solid #d9d9d9;
  border-radius: 3px; font-size: 12px; color: #262626; box-sizing: border-box;
}
.edit-cell-input:focus { outline: none; border-color: #1677ff; }
.edit-cell-computed { background: #f5f5f5; color: #1677ff; cursor: not-allowed; }
.edit-cell-readonly { color: #bfbfbf; font-size: 12px; }
.edit-cell-wrap { position: relative; display: flex; align-items: center; }
.edit-cell-wrap .edit-cell-input { padding-right: 18px; }
.edit-cell-suffix { position: absolute; right: 4px; font-size: 11px; color: #8c8c8c; pointer-events: none; }

/* 明细行浮层 */
.row-popup-mask { position: fixed; inset: 0; z-index: 2000; }
.row-popup {
  position: fixed;
  background: #fff; border: 1px solid #e8e8e8; border-radius: 8px;
  padding: 12px 14px; min-width: 340px; max-width: 420px;
  box-shadow: 0 6px 24px rgba(0,0,0,0.14); z-index: 2001;
}
.popup-section-label {
  grid-column: 1 / -1; font-size: 11px; font-weight: 600;
  color: #1677ff; padding: 4px 0 2px; border-bottom: 1px solid #f0f0f0; margin-bottom: 2px;
  display: flex; align-items: center; gap: 8px;
}
.popup-section-label-2 { margin-top: 6px; }
.btn-apply-sign {
  font-size: 11px; color: #fff; background: #1677ff; border: none;
  border-radius: 3px; padding: 1px 8px; cursor: pointer; font-weight: normal;
  white-space: nowrap; transition: background 0.2s;
}
.btn-apply-sign:hover { background: #4096ff; }
.popup-title {
  display: flex; align-items: center; gap: 8px;
  font-size: 13px; font-weight: 600; color: #262626;
  margin-bottom: 10px; padding-bottom: 8px; border-bottom: 1px solid #f0f0f0;
}
.popup-close {
  margin-left: auto; width: 22px; height: 22px; border: none; background: #f5f5f5;
  border-radius: 4px; color: #8c8c8c; font-size: 14px; cursor: pointer; line-height: 1;
  transition: all 0.2s;
}
.popup-close:hover { background: #ff4d4f; color: #fff; }
.popup-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 8px; margin-bottom: 12px; }
.popup-item { display: flex; flex-direction: column; gap: 3px; }
.popup-item-full { grid-column: 1 / -1; }
.pk { font-size: 11px; color: #8c8c8c; }
.popup-input {
  padding: 5px 8px; border: 1px solid #d9d9d9; border-radius: 4px;
  font-size: 12px; color: #262626; background: #fff; width: 100%; box-sizing: border-box;
}
.popup-input:focus { outline: none; border-color: #1677ff; box-shadow: 0 0 0 2px rgba(22,119,255,0.1); }
.popup-foot { display: flex; align-items: center; justify-content: flex-end; gap: 8px; border-top: 1px solid #f0f0f0; padding-top: 10px; }
.popup-toast { font-size: 12px; margin-right: auto; }
.pt-success { color: #52c41a; }
.pt-error   { color: #ff4d4f; }
.popup-btn-cancel {
  padding: 5px 14px; background: #fff; border: 1px solid #d9d9d9; border-radius: 4px;
  color: #595959; font-size: 12px; cursor: pointer; transition: all 0.2s;
}
.popup-btn-cancel:hover { border-color: #1677ff; color: #1677ff; }
.popup-btn-save {
  padding: 5px 14px; background: #1677ff; border: none; border-radius: 4px;
  color: #fff; font-size: 12px; cursor: pointer; transition: background 0.2s;
}
.popup-btn-save:hover:not(:disabled) { background: #4096ff; }
.popup-btn-save:disabled { opacity: 0.5; cursor: not-allowed; }
.detail-row td { padding: 0; background: #fafafa !important; }
.detail-cell { padding: 0 !important; }
.detail-table { width: 100%; border-collapse: collapse; font-size: 12px; table-layout: fixed; }
.detail-table th, .detail-table td { padding: 5px 6px; border-bottom: 1px solid #f0f0f0; border-right: 1px solid #f0f0f0; text-align: left; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.detail-table th:last-child, .detail-table td:last-child { border-right: none; }
.detail-table thead th { background: #f5f5f5; color: #8c8c8c; font-weight: 500; border-right: 1px solid #e8e8e8; }
.detail-table tbody td { color: #595959; background: #fafafa; }
/* 各列固定宽度 */
.detail-table th:nth-child(1),  .detail-table td:nth-child(1)  { width: 40px; }
.detail-table th:nth-child(2),  .detail-table td:nth-child(2)  { width: 120px; }
.detail-table th:nth-child(3),  .detail-table td:nth-child(3)  { width: 80px; }
.detail-table th:nth-child(4),  .detail-table td:nth-child(4)  { width: 50px; }
.detail-table th:nth-child(5),  .detail-table td:nth-child(5)  { width: 40px; }
.detail-table th:nth-child(6),  .detail-table td:nth-child(6)  { width: 55px; }
.detail-table th:nth-child(7),  .detail-table td:nth-child(7)  { width: 65px; }
.detail-table th:nth-child(8),  .detail-table td:nth-child(8)  { width: 80px; }
.detail-table th:nth-child(9),  .detail-table td:nth-child(9)  { width: 65px; }
.detail-table th:nth-child(10), .detail-table td:nth-child(10) { width: 50px; }
.detail-table th:nth-child(11), .detail-table td:nth-child(11) { width: 80px; }
.detail-table th:nth-child(12), .detail-table td:nth-child(12) { width: 60px; }
.detail-table th:nth-child(13), .detail-table td:nth-child(13) { width: 70px; }
.detail-table th:nth-child(14), .detail-table td:nth-child(14) { width: 50px; }
/* input 撑满单元格但不改变列宽 */
.detail-table td input.edit-cell-input { width: 100%; box-sizing: border-box; }

.pagination { display: flex; align-items: center; justify-content: center; gap: 12px; padding: 8px; background: #fff; border-top: 1px solid #f0f0f0; flex-shrink: 0; }
.pagination button { padding: 4px 12px; background: #fff; border: 1px solid #d9d9d9; border-radius: 4px; color: #595959; font-size: 13px; cursor: pointer; transition: all 0.2s; }
.pagination button:disabled { opacity: 0.4; cursor: not-allowed; }
.pagination button:hover:not(:disabled) { border-color: #1677ff; color: #1677ff; }
.pagination span { color: #595959; font-size: 13px; min-width: 60px; text-align: center; }
</style>
