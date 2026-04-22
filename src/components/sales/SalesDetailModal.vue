<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue';
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

/** 0.13 → "13%" ，非数字原样返回 */
function formatField(field: string, val: string): string {
  if (field === '税率' && val) {
    const n = parseFloat(val);
    if (!isNaN(n)) return `${Math.round(n * 100)}%`;
  }
  return val || '—';
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') emit('close');
}
onMounted(() => window.addEventListener('keydown', onKeydown));
onUnmounted(() => window.removeEventListener('keydown', onKeydown));
</script>

<template>
  <div class="modal-mask">
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
                {{ copiedKey === field ? '已复制 ✓' : formatField(field, row[field]) }}
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
  background: rgba(0,0,0,0.45);
  display: flex; align-items: center; justify-content: center;
  z-index: 1000;
}
.modal {
  background: #fff; border-radius: 10px;
  width: 820px; max-width: 95vw; max-height: 85vh;
  display: flex; flex-direction: column;
  box-shadow: 0 8px 32px rgba(0,0,0,0.12);
}
.modal-head {
  display: flex; align-items: center; justify-content: space-between;
  padding: 16px 20px; border-bottom: 1px solid #f0f0f0; flex-shrink: 0;
}
.head-left { display: flex; align-items: baseline; gap: 10px; }
.modal-title { font-size: 16px; font-weight: 600; color: #262626; }
.modal-sub { font-size: 13px; color: #8c8c8c; }
.head-actions { display: flex; align-items: center; gap: 8px; }

.btn-edit {
  padding: 6px 16px; background: #1677ff; border: none;
  border-radius: 5px; color: #fff; font-size: 13px; cursor: pointer; transition: background 0.2s;
}
.btn-edit:hover { background: #4096ff; }
.btn-delete {
  padding: 6px 16px; background: #fff2f0; border: 1px solid #ffccc7;
  border-radius: 5px; color: #ff4d4f; font-size: 13px; cursor: pointer; transition: all 0.2s;
}
.btn-delete:hover { background: #ff4d4f; color: #fff; }
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

.copy-hint {
  font-size: 11px; color: #bfbfbf; text-align: right; margin-top: -8px;
}

.group-label {
  font-size: 12px; color: #1677ff; font-weight: 600;
  letter-spacing: 0.5px; margin-bottom: 10px;
  padding-bottom: 6px; border-bottom: 1px solid #f0f0f0;
}
.detail-grid {
  display: grid; grid-template-columns: repeat(3, 1fr); gap: 10px 16px;
}
.detail-item {
  display: flex; flex-direction: column; gap: 3px;
  padding: 8px 10px; background: #fafafa; border-radius: 6px;
  border: 1px solid #f0f0f0;
  cursor: default; transition: all 0.15s; user-select: none;
}
.detail-item:hover {
  border-color: #d9d9d9;
}
.detail-item.copied {
  border-color: #52c41a;
  background: #f6ffed;
}
.detail-key { font-size: 11px; color: #8c8c8c; }
.detail-val { font-size: 13px; color: #262626; word-break: break-all; }
.detail-val.empty { color: #bfbfbf; font-style: italic; }
.detail-item.copied .detail-val { color: #52c41a; }

.copy-toast {
  position: fixed;
  bottom: 40px;
  left: 50%;
  transform: translateX(-50%);
  background: #52c41a;
  color: #fff;
  padding: 8px 24px;
  border-radius: 20px;
  font-size: 13px;
  z-index: 9999;
  box-shadow: 0 4px 12px rgba(0,0,0,0.15);
  pointer-events: none;
}
/* stylelint-disable selector-pseudo-class-no-unknown */
.slide-up-enter-active, .slide-up-leave-active { transition: all 0.25s ease; }
.slide-up-enter-from, .slide-up-leave-to { opacity: 0; transform: translateX(-50%) translateY(10px); }
</style>
