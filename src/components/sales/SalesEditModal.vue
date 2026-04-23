<script setup lang="ts">
import { onMounted, onUnmounted, computed, ref } from 'vue';
import { SALES_FORM_GROUPS, COMPUTED_COLUMNS } from '@/constants/salesColumns';
import type { SalesRow } from '@/constants/salesColumns';

const props = defineProps<{
  mode: 'add' | 'edit';
  form: SalesRow;
  isSaving: boolean;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'save'): void;
}>();

const showConfirm = ref(false);
function tryClose() { showConfirm.value = true; }
function confirmClose() { showConfirm.value = false; emit('close'); }

/** 税率回显：0.13 → "13" */
const taxRateDisplay = computed(() => {
  const v = props.form['税率'];
  if (!v) return '';
  const n = parseFloat(v);
  return isNaN(n) ? v : String(Math.round(n * 100));
});

/** 用户输入税率时：20 → 存 0.2 */
function onTaxRateInput(e: Event) {
  const raw = (e.target as HTMLInputElement).value;
  const n = parseFloat(raw);
  props.form['税率'] = raw === '' ? '' : isNaN(n) ? raw : String(n / 100);
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    if (showConfirm.value) { showConfirm.value = false; return; }
    tryClose();
  }
}
onMounted(() => window.addEventListener('keydown', onKeydown));
onUnmounted(() => window.removeEventListener('keydown', onKeydown));
</script>

<template>
  <div class="modal-mask">
    <div class="modal">
      <div class="modal-head">
        <span class="modal-title">{{ mode === 'add' ? '新增记录' : '编辑记录' }}</span>
        <button class="close-btn" @click="tryClose">×</button>
      </div>

      <div class="modal-body">
        <div v-for="group in SALES_FORM_GROUPS" :key="group.label" class="form-group">
          <div class="group-label">{{ group.label }}</div>
          <div class="form-grid">
            <div v-for="field in group.fields" :key="field" class="form-item">
              <label>
                {{ field }}
                <span v-if="COMPUTED_COLUMNS.has(field)" class="computed-tag">自动计算</span>
                <span v-if="field === '税率'" class="computed-tag">输入整数如 13</span>
              </label>
              <!-- 税率单独处理：显示百分比整数，存储小数 -->
              <div v-if="field === '税率'" class="input-wrap">
                <input
                  :value="taxRateDisplay"
                  placeholder="如 13"
                  type="number"
                  min="0"
                  max="100"
                  @input="onTaxRateInput"
                />
                <span class="input-suffix">%</span>
              </div>
              <input
                v-else
                v-model="form[field]"
                :placeholder="COMPUTED_COLUMNS.has(field) ? '自动计算' : field"
                :readonly="COMPUTED_COLUMNS.has(field)"
                :class="{ 'input-computed': COMPUTED_COLUMNS.has(field) }"
              />
            </div>
          </div>
        </div>
      </div>

      <div class="modal-foot">
        <button class="btn-cancel" @click="tryClose">取消</button>
        <button class="btn-save" :disabled="isSaving" @click="emit('save')">
          {{ isSaving ? '保存中...' : '保存' }}
        </button>
      </div>
    </div>

    <!-- 关闭确认 -->
    <transition name="fade">
      <div v-if="showConfirm" class="confirm-mask">
        <div class="confirm-dialog">
          <div class="confirm-title">⚠️ 确认关闭</div>
          <div class="confirm-msg">当前修改尚未保存，确认关闭？</div>
          <div class="confirm-actions">
            <button class="btn-back" @click="showConfirm = false">继续编辑</button>
            <button class="btn-ok" @click="confirmClose">确认关闭</button>
          </div>
        </div>
      </div>
    </transition>
  </div>
</template>

<style scoped>
.modal-mask {
  position: fixed; inset: 0; background: rgba(0,0,0,0.45);
  display: flex; align-items: center; justify-content: center; z-index: 1100;
}
.modal {
  background: #fff; border-radius: 10px;
  width: 780px; max-width: 95vw; max-height: 85vh;
  display: flex; flex-direction: column;
  box-shadow: 0 8px 32px rgba(0,0,0,0.12);
}
.modal-head {
  display: flex; align-items: center; justify-content: space-between;
  padding: 16px 20px; border-bottom: 1px solid #f0f0f0; flex-shrink: 0;
}
.modal-title { font-size: 16px; font-weight: 600; color: #262626; }
.close-btn {
  width: 28px; height: 28px; background: #f5f5f5; border: none;
  border-radius: 5px; color: #8c8c8c; font-size: 18px; cursor: pointer;
  line-height: 1; transition: all 0.2s;
}
.close-btn:hover { background: #ff4d4f; color: #fff; }

.modal-body {
  flex: 1; overflow-y: auto; padding: 16px 20px;
  display: flex; flex-direction: column; gap: 20px;
}
.group-label {
  font-size: 12px; color: #1677ff; font-weight: 600;
  letter-spacing: 0.5px; margin-bottom: 10px;
  padding-bottom: 6px; border-bottom: 1px solid #f0f0f0;
}
.form-grid {
  display: grid; grid-template-columns: repeat(3, 1fr); gap: 10px 16px;
}
.form-item { display: flex; flex-direction: column; gap: 4px; }
.form-item label { font-size: 12px; color: #8c8c8c; display: flex; align-items: center; gap: 4px; }
.form-item input {
  padding: 7px 10px; background: #fff;
  border: 1px solid #d9d9d9; border-radius: 4px; color: #262626; font-size: 13px;
}
.form-item input:focus { outline: none; border-color: #1677ff; box-shadow: 0 0 0 2px rgba(22,119,255,0.1); }
.input-wrap { position: relative; display: flex; align-items: center; }
.input-wrap input { width: 100%; padding-right: 28px; }
.input-suffix {
  position: absolute; right: 10px;
  font-size: 13px; color: #8c8c8c; pointer-events: none;
}
.input-computed {
  background: #f5f5f5 !important; color: #1677ff !important;
  cursor: not-allowed; border-color: #f0f0f0 !important;
}
.computed-tag {
  font-size: 10px; color: #1677ff; background: #e6f4ff;
  border: 1px solid #91caff; border-radius: 3px; padding: 1px 5px; font-weight: normal;
}
.modal-foot {
  display: flex; justify-content: flex-end; gap: 10px;
  padding: 14px 20px; border-top: 1px solid #f0f0f0; flex-shrink: 0;
}
.btn-cancel {
  padding: 8px 20px; background: #fff; border: 1px solid #d9d9d9; border-radius: 6px;
  color: #595959; font-size: 14px; cursor: pointer; transition: all 0.2s;
}
.btn-cancel:hover { border-color: #1677ff; color: #1677ff; }
.btn-save {
  padding: 8px 24px; background: #1677ff; border: none; border-radius: 6px;
  color: #fff; font-size: 14px; cursor: pointer; transition: background 0.2s;
}
.btn-save:hover:not(:disabled) { background: #4096ff; }
.btn-save:disabled { opacity: 0.5; cursor: not-allowed; }
.confirm-mask {
  position: fixed; inset: 0; background: rgba(0,0,0,0.5);
  display: flex; align-items: center; justify-content: center; z-index: 10;
}
.confirm-dialog {
  background: #fff; border-radius: 10px; padding: 24px 28px; width: 360px;
  box-shadow: 0 8px 32px rgba(0,0,0,0.15);
}
.confirm-title { font-size: 16px; font-weight: 600; color: #262626; margin-bottom: 10px; }
.confirm-msg { font-size: 14px; color: #595959; margin-bottom: 20px; }
.confirm-actions { display: flex; gap: 10px; justify-content: flex-end; }
.btn-back {
  padding: 7px 18px; background: #fff; border: 1px solid #d9d9d9; border-radius: 6px;
  color: #595959; font-size: 14px; cursor: pointer; transition: all 0.2s;
}
.btn-back:hover { border-color: #1677ff; color: #1677ff; }
.btn-ok {
  padding: 7px 18px; background: #ff4d4f; border: none; border-radius: 6px;
  color: #fff; font-size: 14px; cursor: pointer;
}
.btn-ok:hover { background: #ff7875; }
.fade-enter-active, .fade-leave-active { transition: opacity 0.2s; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
</style>
