<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import type { QuoteItem } from '../../constants/columns';

interface PriceHistory {
  id: number;
  item_uuid: string;
  cost_price: number | null;
  tax_rate: string | null;
  supplier: string | null;
  date: string | null;
  created_at: string;
}

const props = defineProps<{ item: QuoteItem }>();
const emit  = defineEmits<{ (e: 'close'): void; (e: 'saved'): void }>();

const isSaving   = ref(false);
const showHistory = ref(false);
const history    = ref<PriceHistory[]>([]);
const historyLoading = ref(false);
const toast      = ref('');
const toastOk    = ref(true);

const form = reactive({
  unit:       props.item.unit,
  supplier:   props.item.supplier,
  tax_rate:   props.item.tax_rate,
  cost_price: props.item.cost_price > 0 ? String(props.item.cost_price) : '',
});

async function loadHistory() {
  historyLoading.value = true;
  try {
    history.value = await invoke<PriceHistory[]>('query_price_history', {
      itemUuid: props.item.uuid,
    });
  } finally {
    historyLoading.value = false;
  }
}

async function toggleHistory() {
  showHistory.value = !showHistory.value;
  if (showHistory.value && history.value.length === 0) await loadHistory();
}

async function save() {
  const cp = parseFloat(form.cost_price);
  if (isNaN(cp) || cp <= 0) { showToast('成本单价必须大于 0', false); return; }
  isSaving.value = true;
  try {
    await invoke('update_quote_item', {
      id:        props.item.id,
      unit:      form.unit.trim(),
      supplier:  form.supplier.trim(),
      taxRate:   form.tax_rate.trim(),
      costPrice: cp,
    });
    showToast('保存成功', true);
    setTimeout(() => emit('saved'), 800);
  } catch (e) {
    showToast(`保存失败: ${e}`, false);
  } finally {
    isSaving.value = false;
  }
}

function showToast(msg: string, ok: boolean) {
  toast.value  = msg;
  toastOk.value = ok;
  if (ok) setTimeout(() => (toast.value = ''), 2000);
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') emit('close');
}
onMounted(() => window.addEventListener('keydown', onKeydown));
import { onUnmounted } from 'vue';
onUnmounted(() => window.removeEventListener('keydown', onKeydown));
</script>

<template>
  <div class="modal-mask" @click.self="emit('close')">
    <div class="modal">
      <!-- 头部 -->
      <div class="modal-head">
        <div class="head-info">
          <span class="modal-title">编辑报价物品</span>
          <span class="goods-tag">{{ item.goods_name }}</span>
          <span class="spec-tag">{{ item.spec }}</span>
        </div>
        <button class="close-btn" @click="emit('close')">×</button>
      </div>

      <!-- 表单 -->
      <div class="modal-body">
        <div class="form-grid">
          <div class="form-item">
            <label>单位</label>
            <input v-model="form.unit" class="inp" placeholder="单位" />
          </div>
          <div class="form-item">
            <label>供应商</label>
            <input v-model="form.supplier" class="inp" placeholder="供应商" />
          </div>
          <div class="form-item">
            <label>税率</label>
            <input v-model="form.tax_rate" class="inp" placeholder="如 0.13 或 13%" />
          </div>
          <div class="form-item">
            <label>成本单价（含税）<span class="required">*</span></label>
            <input v-model="form.cost_price" class="inp" placeholder="成本单价" type="number" min="0" step="0.01" />
          </div>
        </div>

        <!-- 历史价格 -->
        <div class="history-section">
          <button class="btn-history" @click="toggleHistory">
            {{ showHistory ? '▲ 收起历史价格' : '▼ 查看历史价格' }}
          </button>
          <div v-if="showHistory" class="history-wrap">
            <div v-if="historyLoading" class="history-empty">加载中...</div>
            <div v-else-if="history.length === 0" class="history-empty">暂无历史记录</div>
            <table v-else class="history-table">
              <thead>
                <tr>
                  <th>日期</th>
                  <th>成本单价</th>
                  <th>税率</th>
                  <th>供应商</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="h in history" :key="h.id">
                  <td>{{ h.date ?? '—' }}</td>
                  <td class="num-col">{{ h.cost_price != null ? h.cost_price.toFixed(2) : '—' }}</td>
                  <td>{{ h.tax_rate ?? '—' }}</td>
                  <td>{{ h.supplier ?? '—' }}</td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </div>

      <!-- 底部 -->
      <div class="modal-foot">
        <span v-if="toast" :class="['foot-toast', toastOk ? 'ft-ok' : 'ft-err']">{{ toast }}</span>
        <button class="btn-cancel" @click="emit('close')">取消</button>
        <button class="btn-save" :disabled="isSaving" @click="save">
          {{ isSaving ? '保存中...' : '保存' }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-mask { position: fixed; inset: 0; background: rgba(0,0,0,0.45); display: flex; align-items: center; justify-content: center; z-index: 1000; }
.modal { background: #fff; border-radius: 10px; width: 520px; max-height: 80vh; display: flex; flex-direction: column; box-shadow: 0 8px 32px rgba(0,0,0,0.14); }
.modal-head { display: flex; align-items: center; justify-content: space-between; padding: 16px 20px; border-bottom: 1px solid #f0f0f0; flex-shrink: 0; }
.head-info { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; }
.modal-title { font-size: 15px; font-weight: 600; color: #262626; }
.goods-tag { padding: 2px 10px; background: #e6f4ff; color: #1677ff; border-radius: 10px; font-size: 12px; border: 1px solid #91caff; }
.spec-tag { padding: 2px 10px; background: #f5f5f5; color: #595959; border-radius: 10px; font-size: 12px; border: 1px solid #e8e8e8; }
.close-btn { width: 28px; height: 28px; background: #f5f5f5; border: none; border-radius: 6px; color: #8c8c8c; font-size: 18px; cursor: pointer; line-height: 1; flex-shrink: 0; }
.close-btn:hover { background: #ff4d4f; color: #fff; }

.modal-body { flex: 1; overflow-y: auto; padding: 20px; }
.form-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 14px; margin-bottom: 16px; }
.form-item { display: flex; flex-direction: column; gap: 5px; }
.form-item-full { grid-column: 1 / -1; }
.form-item label { font-size: 12px; color: #8c8c8c; }
.required { color: #ff4d4f; margin-left: 2px; }
.inp { padding: 7px 10px; border: 1px solid #d9d9d9; border-radius: 6px; font-size: 13px; color: #262626; background: #fff; }
.inp:focus { outline: none; border-color: #1677ff; box-shadow: 0 0 0 2px rgba(22,119,255,0.1); }

.history-section { border-top: 1px solid #f0f0f0; padding-top: 12px; }
.btn-history { background: none; border: none; color: #1677ff; font-size: 12px; cursor: pointer; padding: 0; }
.btn-history:hover { color: #4096ff; }
.history-wrap { margin-top: 10px; }
.history-empty { font-size: 12px; color: #bfbfbf; text-align: center; padding: 12px 0; }
.history-table { width: 100%; border-collapse: collapse; font-size: 12px; }
.history-table th, .history-table td { padding: 6px 8px; border-bottom: 1px solid #f0f0f0; text-align: left; white-space: nowrap; }
.history-table thead th { background: #fafafa; color: #8c8c8c; font-weight: 500; }
.history-table tbody td { color: #595959; }
.time-col { color: #8c8c8c; font-size: 11px; }
.num-col { text-align: right; font-variant-numeric: tabular-nums; color: #1677ff; }

.modal-foot { display: flex; align-items: center; justify-content: flex-end; gap: 8px; padding: 14px 20px; border-top: 1px solid #f0f0f0; flex-shrink: 0; }
.foot-toast { font-size: 12px; margin-right: auto; }
.ft-ok  { color: #52c41a; }
.ft-err { color: #ff4d4f; }
.btn-cancel { padding: 6px 18px; background: #fff; border: 1px solid #d9d9d9; border-radius: 6px; color: #595959; font-size: 13px; cursor: pointer; }
.btn-cancel:hover { border-color: #1677ff; color: #1677ff; }
.btn-save { padding: 6px 18px; background: #1677ff; border: none; border-radius: 6px; color: #fff; font-size: 13px; cursor: pointer; }
.btn-save:hover:not(:disabled) { background: #4096ff; }
.btn-save:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
