<script setup lang="ts">
import { useCopyCell } from '@/composables/useCopyCell.ts';
import { SALES_FORM_GROUPS } from '@/constants/salesColumns.ts';
import type { SalesRow } from '@/constants/salesColumns.ts';

defineProps<{ row: SalesRow }>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'edit', row: SalesRow): void;
  (e: 'delete', row: SalesRow): void;
}>();

const { copiedKey, toastVisible, copyCell } = useCopyCell();
</script>

<template>
  <div class="modal-mask" @click.self="emit('close')">
    <div class="modal">
      <div class="modal-head">
        <div class="head-left">
          <span class="modal-title">{{ row['客户'] || '详情' }}</span>
          <span class="modal-sub">{{ row['合同号'] }}</span>
        </div>
        <div class="head-actions">
          <button class="btn-edit" @click="emit('edit', row)">编辑</button>
          <button class="btn-delete" @click="emit('delete', row)">删除</button>
          <button class="close-btn" @click="emit('close')">×</button>
        </div>
      </div>

      <div class="modal-body">
        <p class="copy-hint">双击字段值可复制</p>

        <!-- 复制成功提示 -->
        <transition name="slide-up">
          <div v-if="toastVisible" class="copy-toast">复制成功 ✓</div>
        </transition>
        <div v-for="group in SALES_FORM_GROUPS" :key="group.label" class="detail-group">
          <div class="group-label">{{ group.label }}</div>
          <div class="detail-grid">
            <div
              v-for="field in group.fields"
              :key="field"
              class="detail-item"
              :class="{ copied: copiedKey === field }"
              @dblclick="copyCell(field, row[field])"
            >
              <span class="detail-key">{{ field }}</span>
              <span class="detail-val" :class="{ empty: !row[field] }">
                {{ copiedKey === field ? '已复制 ✓' : (row[field] || '—') }}
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-mask {
  position: fixed; inset: 0;
  background: rgba(0,0,0,0.72);
  display: flex; align-items: center; justify-content: center;
  z-index: 1000;
}
.modal {
  background: #1a1a2e; border-radius: 10px;
  width: 820px; max-width: 95vw; max-height: 85vh;
  display: flex; flex-direction: column;
  box-shadow: 0 20px 60px rgba(0,0,0,0.5);
}
.modal-head {
  display: flex; align-items: center; justify-content: space-between;
  padding: 16px 20px; border-bottom: 1px solid #0f3460; flex-shrink: 0;
}
.head-left { display: flex; align-items: baseline; gap: 10px; }
.modal-title { font-size: 16px; font-weight: 600; color: #fff; }
.modal-sub { font-size: 13px; color: #666; }
.head-actions { display: flex; align-items: center; gap: 8px; }

.btn-edit {
  padding: 6px 16px; background: #4a9eff; border: none;
  border-radius: 5px; color: #fff; font-size: 13px; cursor: pointer; transition: background 0.2s;
}
.btn-edit:hover { background: #3a8eef; }
.btn-delete {
  padding: 6px 16px; background: #2a1010; border: 1px solid #e74c3c44;
  border-radius: 5px; color: #e74c3c; font-size: 13px; cursor: pointer; transition: all 0.2s;
}
.btn-delete:hover { background: #e74c3c; color: #fff; }
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

.copy-hint {
  font-size: 11px; color: #444; text-align: right; margin-top: -8px;
}

.group-label {
  font-size: 12px; color: #4a9eff; font-weight: 600;
  letter-spacing: 0.5px; margin-bottom: 10px;
  padding-bottom: 6px; border-bottom: 1px solid #0f3460;
}
.detail-grid {
  display: grid; grid-template-columns: repeat(3, 1fr); gap: 10px 16px;
}
.detail-item {
  display: flex; flex-direction: column; gap: 3px;
  padding: 8px 10px; background: #16213e; border-radius: 6px;
  border: 1px solid #0f3460;
  cursor: default; transition: all 0.15s; user-select: none;
}
.detail-item:hover {
  border-color: #1a4a7a;
}
.detail-item.copied {
  border-color: #27ae60;
  background: #0d2a1a;
}
.detail-key { font-size: 11px; color: #666; }
.detail-val { font-size: 13px; color: #ddd; word-break: break-all; }
.detail-val.empty { color: #444; font-style: italic; }
.detail-item.copied .detail-val { color: #27ae60; }

.copy-toast {
  position: fixed;
  bottom: 40px;
  left: 50%;
  transform: translateX(-50%);
  background: #27ae60;
  color: #fff;
  padding: 8px 24px;
  border-radius: 20px;
  font-size: 13px;
  z-index: 9999;
  box-shadow: 0 4px 12px rgba(0,0,0,0.3);
  pointer-events: none;
}
/* stylelint-disable selector-pseudo-class-no-unknown */
.slide-up-enter-active, .slide-up-leave-active { transition: all 0.25s ease; }
.slide-up-enter-from, .slide-up-leave-to { opacity: 0; transform: translateX(-50%) translateY(10px); }
</style>
