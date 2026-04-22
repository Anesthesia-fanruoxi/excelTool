<script setup lang="ts">
defineEmits<{
  (e: 'cancel'): void;
  (e: 'confirm'): void;
}>();
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
  position: fixed; inset: 0; background: rgba(0,0,0,0.72);
  display: flex; align-items: center; justify-content: center; z-index: 1200;
}
.confirm-dialog {
  background: #1a1a2e; border: 1px solid #0f3460; border-radius: 10px;
  padding: 28px 32px; width: 380px; box-shadow: 0 20px 60px rgba(0,0,0,0.5);
}
.confirm-title { font-size: 17px; font-weight: 600; color: #fff; margin-bottom: 12px; }
.confirm-msg   { font-size: 14px; color: #aaa; line-height: 1.6; margin-bottom: 24px; }
.confirm-actions { display: flex; gap: 10px; justify-content: flex-end; }
.btn-cancel {
  padding: 8px 20px; background: #0f3460; border: none; border-radius: 6px;
  color: #ccc; font-size: 14px; cursor: pointer; transition: all 0.2s;
}
.btn-cancel:hover { background: #1a4a7a; color: #fff; }
.btn-ok {
  padding: 8px 20px; background: #e74c3c; border: none; border-radius: 6px;
  color: #fff; font-size: 14px; cursor: pointer; transition: background 0.2s;
}
.btn-ok:hover { background: #c0392b; }
</style>
