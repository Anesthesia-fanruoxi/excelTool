<script setup lang="ts">
import { ref, reactive } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import type { QuoteItem } from '../../constants/columns';

interface ContractItem {
  item_uuid: string;
  product_name: string;  // 可修改
  spec: string;
  unit: string;
  supplier: string;
  quantity: string;
  unit_price: string;
  remark: string;
}

const props = defineProps<{ selectedItems: QuoteItem[] }>();
const emit  = defineEmits<{ (e: 'close'): void; (e: 'saved'): void }>();

const isSaving = ref(false);
const toast    = ref('');
const toastOk  = ref(true);

const basic = reactive({
  contract_no:  '',
  customer:     '',
  sale_date:    '',
  project_name: '',
});

// 从报价物品初始化明细行
const rows = ref<ContractItem[]>(
  props.selectedItems.map(q => ({
    item_uuid:    q.uuid,
    product_name: q.goods_name,
    spec:         q.spec,
    unit:         q.unit,
    supplier:     q.supplier,
    quantity:     '',
    unit_price:   '',
    remark:       '',
  }))
);

function removeRow(idx: number) {
  rows.value.splice(idx, 1);
}

async function save() {
  if (!basic.contract_no.trim()) { showToast('请填写合同号', false); return; }
  if (!basic.customer.trim())    { showToast('请填写客户', false); return; }
  if (rows.value.length === 0)   { showToast('至少保留一条明细', false); return; }

  for (const r of rows.value) {
    if (!r.quantity || isNaN(parseFloat(r.quantity))) {
      showToast(`「${r.product_name}」数量未填写`, false); return;
    }
    if (!r.unit_price || isNaN(parseFloat(r.unit_price))) {
      showToast(`「${r.product_name}」销售单价未填写`, false); return;
    }
  }

  isSaving.value = true;
  try {
    const items = rows.value.map(r => ({
      product_name: r.product_name.trim(),
      spec:         r.spec,
      quantity:     parseFloat(r.quantity),
      unit:         r.unit,
      unit_price:   parseFloat(r.unit_price),
      supplier:     r.supplier,
      remark:       r.remark.trim(),
      item_uuid:    r.item_uuid || null,
    }));

    await invoke('create_contract', {
      contractNo:  basic.contract_no.trim(),
      customer:    basic.customer.trim(),
      saleDate:    basic.sale_date.trim(),
      projectName: basic.project_name.trim(),
      items,
    });

    showToast('合同创建成功', true);
    setTimeout(() => emit('saved'), 800);
  } catch (e) {
    showToast(`创建失败: ${e}`, false);
  } finally {
    isSaving.value = false;
  }
}

function showToast(msg: string, ok: boolean) {
  toast.value  = msg;
  toastOk.value = ok;
  if (ok) setTimeout(() => (toast.value = ''), 1500);
}
</script>

<template>
  <div class="modal-mask" @click.self="emit('close')">
    <div class="modal">
      <div class="modal-head">
        <span class="modal-title">创建合同</span>
        <span class="sel-badge">已选 {{ rows.length }} 条物品</span>
        <button class="close-btn" @click="emit('close')">×</button>
      </div>

      <div class="modal-body">
        <!-- 基本信息 -->
        <div class="section-label">合同基本信息</div>
        <div class="basic-grid">
          <div class="form-item">
            <label>合同号 <span class="req">*</span></label>
            <input v-model="basic.contract_no" class="inp" placeholder="请输入合同号" />
          </div>
          <div class="form-item">
            <label>客户 <span class="req">*</span></label>
            <input v-model="basic.customer" class="inp" placeholder="客户名称" />
          </div>
          <div class="form-item">
            <label>销售日期</label>
            <input v-model="basic.sale_date" class="inp" placeholder="如 2026-04-23" />
          </div>
          <div class="form-item">
            <label>项目名称</label>
            <input v-model="basic.project_name" class="inp" placeholder="项目名称" />
          </div>
        </div>

        <!-- 明细表 -->
        <div class="section-label" style="margin-top:16px">
          产品明细
          <span class="hint">产品名称可修改，其他信息来自报价单</span>
        </div>
        <div class="table-wrap">
          <table class="detail-table">
            <thead>
              <tr>
                <th class="col-name">产品名称</th>
                <th>规格</th>
                <th>单位</th>
                <th>供应商</th>
                <th class="col-num">数量 <span class="req">*</span></th>
                <th class="col-num">销售单价 <span class="req">*</span></th>
                <th class="col-remark">备注</th>
                <th class="col-del"></th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="(row, idx) in rows" :key="idx">
                <td>
                  <input v-model="row.product_name" class="cell-inp" placeholder="产品名称" />
                </td>
                <td class="readonly">{{ row.spec }}</td>
                <td class="readonly">{{ row.unit }}</td>
                <td class="readonly">{{ row.supplier }}</td>
                <td>
                  <input v-model="row.quantity" class="cell-inp num-inp" placeholder="0" type="number" min="0" />
                </td>
                <td>
                  <input v-model="row.unit_price" class="cell-inp num-inp" placeholder="0.00" type="number" min="0" step="0.01" />
                </td>
                <td>
                  <input v-model="row.remark" class="cell-inp" placeholder="备注" />
                </td>
                <td class="col-del">
                  <button class="btn-del" @click="removeRow(idx)" title="移除">×</button>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <div class="modal-foot">
        <span v-if="toast" :class="['foot-toast', toastOk ? 'ft-ok' : 'ft-err']">{{ toast }}</span>
        <button class="btn-cancel" @click="emit('close')">取消</button>
        <button class="btn-save" :disabled="isSaving || rows.length === 0" @click="save">
          {{ isSaving ? '创建中...' : '确认创建' }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-mask { position: fixed; inset: 0; background: rgba(0,0,0,0.45); display: flex; align-items: center; justify-content: center; z-index: 1000; }
.modal { background: #fff; border-radius: 10px; width: 900px; max-width: 96vw; max-height: 88vh; display: flex; flex-direction: column; box-shadow: 0 8px 32px rgba(0,0,0,0.14); }

.modal-head { display: flex; align-items: center; gap: 10px; padding: 16px 20px; border-bottom: 1px solid #f0f0f0; flex-shrink: 0; }
.modal-title { font-size: 15px; font-weight: 600; color: #262626; }
.sel-badge { padding: 2px 10px; background: #e6f4ff; color: #1677ff; border-radius: 10px; font-size: 12px; border: 1px solid #91caff; }
.close-btn { margin-left: auto; width: 28px; height: 28px; background: #f5f5f5; border: none; border-radius: 6px; color: #8c8c8c; font-size: 18px; cursor: pointer; }
.close-btn:hover { background: #ff4d4f; color: #fff; }

.modal-body { flex: 1; overflow-y: auto; padding: 16px 20px; }
.section-label { font-size: 12px; font-weight: 600; color: #1677ff; margin-bottom: 10px; display: flex; align-items: center; gap: 8px; }
.hint { font-size: 11px; color: #bfbfbf; font-weight: normal; }

.basic-grid { display: grid; grid-template-columns: 1fr 1fr 1fr 1fr; gap: 12px; margin-bottom: 4px; }
.form-item { display: flex; flex-direction: column; gap: 4px; }
.form-item label { font-size: 12px; color: #8c8c8c; }
.req { color: #ff4d4f; }
.inp { padding: 6px 10px; border: 1px solid #d9d9d9; border-radius: 6px; font-size: 13px; color: #262626; }
.inp:focus { outline: none; border-color: #1677ff; box-shadow: 0 0 0 2px rgba(22,119,255,0.1); }

.table-wrap { overflow-x: auto; }
.detail-table { width: 100%; border-collapse: collapse; font-size: 13px; }
.detail-table th, .detail-table td { padding: 6px 8px; border-bottom: 1px solid #f0f0f0; text-align: left; white-space: nowrap; }
.detail-table thead th { background: #fafafa; color: #8c8c8c; font-size: 12px; font-weight: 500; }
.col-name { min-width: 140px; }
.col-num  { width: 100px; }
.col-remark { min-width: 120px; }
.col-del  { width: 32px; text-align: center; }
.cell-inp { width: 100%; padding: 4px 6px; border: 1px solid #e8e8e8; border-radius: 4px; font-size: 12px; color: #262626; box-sizing: border-box; background: #fff; }
.cell-inp:focus { outline: none; border-color: #1677ff; }
.num-inp { text-align: right; }
.readonly { color: #8c8c8c; font-size: 12px; background: #fafafa; }
.btn-del { width: 22px; height: 22px; border: none; background: #fff2f0; border-radius: 4px; color: #ff4d4f; font-size: 14px; cursor: pointer; line-height: 1; }
.btn-del:hover { background: #ff4d4f; color: #fff; }

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
