<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import type { VaultEntry } from '@/types.ts';

// 销售表标准列定义（顺序与表头一致）
const SALES_COLUMNS = [
  { key: '客户',        label: '客户',        width: 100 },
  { key: '销售日期',    label: '销售日期',    width: 100 },
  { key: '合同号',      label: '合同号',      width: 110 },
  { key: '送货单号',    label: '送货单号',    width: 110 },
  { key: '项目名称',    label: '项目名称',    width: 140 },
  { key: '收货地址',    label: '收货地址',    width: 140 },
  { key: '序号',        label: '序号',        width: 60  },
  { key: '产品名称',    label: '产品名称',    width: 140 },
  { key: '规格',        label: '规格',        width: 100 },
  { key: '特征',        label: '特征',        width: 100 },
  { key: '数量',        label: '数量',        width: 70  },
  { key: '单位',        label: '单位',        width: 70  },
  { key: '单价',        label: '单价',        width: 80  },
  { key: '金额',        label: '金额',        width: 90  },
  { key: '下单人',      label: '下单人',      width: 80  },
  { key: '安装位置',    label: '安装位置',    width: 110 },
  { key: '备注',        label: '备注',        width: 120 },
  { key: '所属年份',    label: '所属年份',    width: 90  },
  { key: '签收人',      label: '签收人',      width: 80  },
  { key: '签收日期',    label: '签收日期',    width: 100 },
  { key: '与客户对账时间', label: '与客户对账时间', width: 130 },
  { key: '状态',        label: '状态',        width: 80  },
  { key: '供应商',      label: '供应商',      width: 100 },
  { key: '初始报价',    label: '初始报价',    width: 90  },
  { key: '税率',        label: '税率',        width: 70  },
  { key: '成本单价(含税)', label: '成本单价(含税)', width: 120 },
  { key: '应付金额',    label: '应付金额',    width: 90  },
  { key: '对账数量',    label: '对账数量',    width: 90  },
  { key: '对账单价',    label: '对账单价',    width: 90  },
  { key: '对账日期',    label: '对账日期',    width: 100 },
  { key: '对账金额',    label: '对账金额',    width: 90  },
  { key: '对账备注',    label: '对账备注',    width: 110 },
  { key: '利润',        label: '利润',        width: 80  },
];

// 数字类型列（右对齐）
const NUMBER_COLS = new Set(['数量','单价','金额','初始报价','成本单价(含税)','应付金额','对账数量','对账单价','对账金额','利润','税率']);

type RowRecord = Record<string, string>;

const isLoading = ref(false);
const errorMsg = ref('');
const allRows = ref<RowRecord[]>([]);
const sourceEntry = ref<VaultEntry | null>(null);

// 筛选
const filterText = ref('');
const filterCol = ref('');  // 空=全列搜索

// 分页
const pageSize = 100;
const currentPage = ref(1);

const filteredRows = computed(() => {
  if (!filterText.value) return allRows.value;
  const kw = filterText.value.toLowerCase();
  return allRows.value.filter(row => {
    if (filterCol.value) {
      return (row[filterCol.value] ?? '').toLowerCase().includes(kw);
    }
    return SALES_COLUMNS.some(col => (row[col.key] ?? '').toLowerCase().includes(kw));
  });
});

const totalPages = computed(() => Math.max(1, Math.ceil(filteredRows.value.length / pageSize)));

const pagedRows = computed(() => {
  const start = (currentPage.value - 1) * pageSize;
  return filteredRows.value.slice(start, start + pageSize);
});

// 从加密库加载销售表数据
async function loadData() {
  isLoading.value = true;
  errorMsg.value = '';
  try {
    const vault = await invoke<{ entries: VaultEntry[] }>('get_vault_status');
    if (vault.entries.length === 0) {
      allRows.value = [];
      return;
    }
    // 取最新一条（后续可做多条选择）
    const entry = vault.entries[vault.entries.length - 1];
    sourceEntry.value = entry;
    allRows.value = mapToRows(entry);
  } catch (e) {
    errorMsg.value = `加载失败: ${e}`;
  } finally {
    isLoading.value = false;
  }
}

// 将 VaultEntry 的 headers+data 映射为对象数组
function mapToRows(entry: VaultEntry): RowRecord[] {
  const headers = entry.headers;
  return entry.data.map(row => {
    const record: RowRecord = {};
    headers.forEach((h, i) => {
      record[h.trim()] = row[i] ?? '';
    });
    return record;
  });
}

function isNumber(colKey: string) {
  return NUMBER_COLS.has(colKey);
}

function prevPage() { if (currentPage.value > 1) currentPage.value--; }
function nextPage() { if (currentPage.value < totalPages.value) currentPage.value++; }

// 筛选重置分页
function onFilter() { currentPage.value = 1; }

onMounted(loadData);

defineExpose({ reload: loadData });
</script>

<template>
  <div class="sales-table-view">
    <!-- 工具栏 -->
    <div class="toolbar">
      <div class="filter-group">
        <select v-model="filterCol" class="col-select" @change="onFilter">
          <option value="">全部列</option>
          <option v-for="col in SALES_COLUMNS" :key="col.key" :value="col.key">
            {{ col.label }}
          </option>
        </select>
        <input
          v-model="filterText"
          class="filter-input"
          placeholder="搜索..."
          @input="onFilter"
        />
      </div>
      <div class="toolbar-right">
        <span class="row-count">
          共 <b>{{ filteredRows.length }}</b> 行
          <template v-if="filterText"> / {{ allRows.length }} 行</template>
        </span>
        <button class="refresh-btn" @click="loadData">刷新</button>
      </div>
    </div>

    <!-- 错误提示 -->
    <div v-if="errorMsg" class="error-bar">{{ errorMsg }}</div>

    <!-- 加载中 -->
    <div v-if="isLoading" class="loading-bar">加载中...</div>

    <!-- 空状态 -->
    <div v-else-if="allRows.length === 0 && !isLoading" class="empty-state">
      <div class="empty-icon">📋</div>
      <p>暂无销售数据</p>
      <p class="empty-hint">请先在「导入」功能中导入 Excel 文件</p>
    </div>

    <!-- 表格 -->
    <div v-else class="table-wrap">
      <table class="sales-table">
        <thead>
          <tr>
            <th class="col-seq">#</th>
            <th
              v-for="col in SALES_COLUMNS"
              :key="col.key"
              :style="{ minWidth: col.width + 'px' }"
            >
              {{ col.label }}
            </th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="(row, ri) in pagedRows" :key="ri">
            <td class="col-seq">{{ (currentPage - 1) * pageSize + ri + 1 }}</td>
            <td
              v-for="col in SALES_COLUMNS"
              :key="col.key"
              :class="{ 'num-cell': isNumber(col.key) }"
            >
              {{ row[col.key] ?? '' }}
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- 分页 -->
    <div v-if="allRows.length > 0" class="pagination">
      <button :disabled="currentPage === 1" @click="prevPage">上一页</button>
      <span>{{ currentPage }} / {{ totalPages }}</span>
      <button :disabled="currentPage >= totalPages" @click="nextPage">下一页</button>
    </div>
  </div>
</template>

<style scoped>
.sales-table-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

/* 工具栏 */
.toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 20px;
  border-bottom: 1px solid #0f3460;
  gap: 12px;
  flex-shrink: 0;
}
.filter-group {
  display: flex;
  gap: 8px;
  align-items: center;
}
.col-select {
  padding: 6px 10px;
  background: #0f3460;
  border: 1px solid #1a4a7a;
  border-radius: 4px;
  color: #ccc;
  font-size: 13px;
  cursor: pointer;
}
.col-select:focus { outline: none; border-color: #4a9eff; }
.filter-input {
  padding: 6px 12px;
  background: #0f3460;
  border: 1px solid #1a4a7a;
  border-radius: 4px;
  color: #fff;
  font-size: 13px;
  width: 200px;
}
.filter-input:focus { outline: none; border-color: #4a9eff; }
.toolbar-right {
  display: flex;
  align-items: center;
  gap: 12px;
}
.row-count { font-size: 13px; color: #888; }
.row-count b { color: #4a9eff; }
.refresh-btn {
  padding: 6px 14px;
  background: #0f3460;
  border: none;
  border-radius: 4px;
  color: #ccc;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
}
.refresh-btn:hover { background: #4a9eff; color: #fff; }

/* 状态栏 */
.error-bar {
  padding: 10px 20px;
  background: #e74c3c22;
  color: #e74c3c;
  font-size: 13px;
  flex-shrink: 0;
}
.loading-bar {
  padding: 10px 20px;
  color: #4a9eff;
  font-size: 13px;
  flex-shrink: 0;
}

/* 空状态 */
.empty-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: #666;
  text-align: center;
}
.empty-icon { font-size: 48px; margin-bottom: 16px; }
.empty-state p { font-size: 15px; margin-bottom: 6px; }
.empty-hint { font-size: 13px; color: #555; }

/* 表格 */
.table-wrap {
  flex: 1;
  overflow: auto;
}
.sales-table {
  border-collapse: collapse;
  font-size: 13px;
  width: max-content;
  min-width: 100%;
}
.sales-table th,
.sales-table td {
  padding: 7px 10px;
  border: 1px solid #0f3460;
  white-space: nowrap;
  text-align: left;
}
.sales-table thead th {
  background: #16213e;
  color: #4a9eff;
  font-weight: 600;
  position: sticky;
  top: 0;
  z-index: 10;
}
.sales-table tbody td {
  color: #ccc;
  background: #1a1a2e;
}
.sales-table tbody tr:hover td {
  background: #16213e;
}
.col-seq {
  width: 46px;
  min-width: 46px !important;
  text-align: center;
  background: #0f3460 !important;
  color: #666 !important;
  position: sticky;
  left: 0;
  z-index: 5;
}
.num-cell {
  text-align: right;
  font-variant-numeric: tabular-nums;
}

/* 分页 */
.pagination {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 16px;
  padding: 10px;
  border-top: 1px solid #0f3460;
  flex-shrink: 0;
}
.pagination button {
  padding: 6px 14px;
  background: #0f3460;
  border: none;
  border-radius: 4px;
  color: #ccc;
  font-size: 13px;
  cursor: pointer;
}
.pagination button:disabled { opacity: 0.4; cursor: not-allowed; }
.pagination button:hover:not(:disabled) { background: #1a4a7a; color: #fff; }
.pagination span { color: #ccc; font-size: 13px; }
</style>
