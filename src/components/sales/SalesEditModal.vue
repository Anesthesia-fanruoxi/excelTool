<script setup lang="ts">
import { SALES_FORM_GROUPS, COMPUTED_COLUMNS } from '@/constants/salesColumns.ts';
import type { SalesRow } from '@/constants/salesColumns.ts';

defineProps<{
  mode: 'add' | 'edit';
  form: SalesRow;
  isSaving: boolean;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'save'): void;
}>();
</script>

<template>
  <div class="modal-mask" @click.self="emit('close')">
    <div class="modal">
      <div class="modal-head">
        <span class="modal-title">{{ mode === 'add' ? '新增记录' : '编辑记录' }}</span>
        <button class="close-btn" @click="emit('close')">×</button>
      </div>

      <div class="modal-body">
        <div v-for="group in SALES_FORM_GROUPS" :key="group.label" class="form-group">
          <div class="group-label">{{ group.label }}</div>
          <div class="form-grid">
            <div v-for="field in group.fields" :key="field" class="form-item">
              <label>
                {{ field }}
                <span v-if="COMPUTED_COLUMNS.has(field)" class="computed-tag">自动计算</span>
              </label>
              <input
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
        <button class="btn-cancel" @click="emit('close')">取消</button>
        <button class="btn-save" :disabled="isSaving" @click="emit('save')">
          {{ isSaving ? '保存中...' : '保存' }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-mask {
  position: fixed; inset: 0; background: rgba(0,0,0,0.72);
  display: flex; align-items: center; justify-content: center; z-index: 1100;
}
.modal {
  background: #1a1a2e; border-radius: 10px;
  width: 780px; max-width: 95vw; max-height: 85vh;
  display: flex; flex-direction: column;
  box-shadow: 0 20px 60px rgba(0,0,0,0.5);
}
.modal-head {
  display: flex; align-items: center; justify-content: space-between;
  padding: 16px 20px; border-bottom: 1px solid #0f3460; flex-shrink: 0;
}
.modal-title { font-size: 16px; font-weight: 600; color: #fff; }
.close-btn {
  width: 28px; height: 28px; background: #0f3460; border: none;
  border-radius: 5px; color: #aaa; font-size: 18px; cursor: pointer;
  line-height: 1; transition: all 0.2s;
}
.close-btn:hover { background: #e74c3c; color: #fff; }

.modal-body {
  flex: 1; overflow-y: auto; padding: 16px 20px;
  display: flex; flex-direction: column; gap: 20px;
}
.group-label {
  font-size: 12px; color: #4a9eff; font-weight: 600;
  letter-spacing: 0.5px; margin-bottom: 10px;
  padding-bottom: 6px; border-bottom: 1px solid #0f3460;
}
.form-grid {
  display: grid; grid-template-columns: repeat(3, 1fr); gap: 10px 16px;
}
.form-item { display: flex; flex-direction: column; gap: 4px; }
.form-item label { font-size: 12px; color: #888; display: flex; align-items: center; gap: 4px; }
.form-item input {
  padding: 7px 10px; background: #0f3460;
  border: 1px solid #1a4a7a; border-radius: 4px; color: #fff; font-size: 13px;
}
.form-item input:focus { outline: none; border-color: #4a9eff; }
.input-computed {
  background: #0a1f35 !important; color: #4a9eff !important;
  cursor: not-allowed; border-color: #0f3460 !important;
}
.computed-tag {
  font-size: 10px; color: #4a9eff; background: #4a9eff22;
  border: 1px solid #4a9eff44; border-radius: 3px; padding: 1px 5px; font-weight: normal;
}
.modal-foot {
  display: flex; justify-content: flex-end; gap: 10px;
  padding: 14px 20px; border-top: 1px solid #0f3460; flex-shrink: 0;
}
.btn-cancel {
  padding: 8px 20px; background: #0f3460; border: none; border-radius: 6px;
  color: #ccc; font-size: 14px; cursor: pointer; transition: all 0.2s;
}
.btn-cancel:hover { background: #1a4a7a; color: #fff; }
.btn-save {
  padding: 8px 24px; background: #4a9eff; border: none; border-radius: 6px;
  color: #fff; font-size: 14px; cursor: pointer; transition: background 0.2s;
}
.btn-save:hover:not(:disabled) { background: #3a8eef; }
.btn-save:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
