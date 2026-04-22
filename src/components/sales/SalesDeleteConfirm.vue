<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue';

const emit = defineEmits<{
  (e: 'cancel'): void;
  (e: 'confirm'): void;
}>();

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') emit('cancel');
}
onMounted(() => window.addEventListener('keydown', onKeydown));
onUnmounted(() => window.removeEventListener('keydown', onKeydown));
</script>

<template>
  <div class="modal-mask" @click.self="$emit('cancel')">
    <div class="confirm-dialog">
      <div class="confirm-title">确认删除</div>
      <div class="confirm-msg">删除后无法恢复，确认删除该条记录？</div>
      <div class="confirm-actions">
        <button class="btn-cancel" @click="$emit('cancel')">取消</button>
        <button class="btn-ok" @click="$emit('confirm')">确认删除</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-mask {
  position: fixed; inset: 0; background: rgba(0,0,0,0.45);
  display: flex; align-items: center; justify-content: center; z-index: 1200;
}
.confirm-dialog {
  background: #fff; border: 1px solid #e8e8e8; border-radius: 10px;
  padding: 28px 32px; width: 380px; box-shadow: 0 8px 32px rgba(0,0,0,0.12);
}
.confirm-title { font-size: 17px; font-weight: 600; color: #262626; margin-bottom: 12px; }
.confirm-msg   { font-size: 14px; color: #595959; line-height: 1.6; margin-bottom: 24px; }
.confirm-actions { display: flex; gap: 10px; justify-content: flex-end; }
.btn-cancel {
  padding: 8px 20px; background: #fff; border: 1px solid #d9d9d9; border-radius: 6px;
  color: #595959; font-size: 14px; cursor: pointer; transition: all 0.2s;
}
.btn-cancel:hover { border-color: #1677ff; color: #1677ff; }
.btn-ok {
  padding: 8px 20px; background: #ff4d4f; border: none; border-radius: 6px;
  color: #fff; font-size: 14px; cursor: pointer; transition: background 0.2s;
}
.btn-ok:hover { background: #ff7875; }
</style>
