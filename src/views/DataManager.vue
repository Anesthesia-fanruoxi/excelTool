<script setup lang="ts">
import { useDataManager } from '../composables/useDataManager';

const {
  FIXED_TABLES,
  isLoading,
  isImporting,
  tableStats,
  flippedCard,
  confirmDialog,
  previewEntry,
  previewPage,
  previewPageSize,
  toastMsg,
  toastType,
  flipCard,
  handlePreview,
  handleClear,
  handleImport,
  confirmAction,
  cancelConfirm,
  getPreviewRows,
  previewTotalPages,
} = useDataManager();
</script>

<template>
  <div class="data-manager">
    <transition name="fade">
      <div v-if="toastMsg" :class="['toast', toastType === 'error' ? 'toast-error' : 'toast-success']">
        {{ toastMsg }}
      </div>
    </transition>

    <div class="top-bar">
      <span class="page-title">数据管理</span>
    </div>

    <div class="content">
      <div v-if="isLoading" class="status-center">
        <span class="loading-text">加载中...</span>
      </div>

      <div v-else class="card-grid">
        <div
          v-for="table in FIXED_TABLES"
          :key="table.key"
          :class="['card-scene', { flipped: flippedCard === table.key }]"
        >
          <!-- 正面 -->
          <div class="card-face card-front" @click="flipCard(table.key)">
            <div class="card-front-icon">{{ table.icon }}</div>
            <div class="card-front-body">
              <div class="card-label">{{ table.label }}</div>
              <div class="card-count">
                <span class="count-num">{{ tableStats[table.key] ?? 0 }}</span>
                <span class="count-unit"> 条数据</span>
              </div>
              <div class="card-desc">{{ table.desc }}</div>
            </div>
            <div class="card-flip-hint">点击查看操作 &rsaquo;</div>
          </div>

          <!-- 背面 -->
          <div class="card-face card-back">
            <div class="back-title">{{ table.label }}</div>
            <div class="back-actions">
              <button class="action-btn btn-preview" @click.stop="handlePreview">
                <span class="action-icon">👁</span>预览数据
              </button>
              <button class="action-btn btn-danger" :disabled="isImporting" @click.stop="handleImport">
                <span class="action-icon">📥</span>{{ isImporting ? '导入中...' : '导入数据' }}
              </button>
              <button class="action-btn btn-danger" @click.stop="handleClear">
                <span class="action-icon">🗑</span>清空数据
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 二次确认 -->
    <div v-if="confirmDialog.show" class="modal-mask" @click.self="cancelConfirm">
      <div class="confirm-dialog">
        <div class="confirm-title">{{ confirmDialog.title }}</div>
        <div class="confirm-msg">{{ confirmDialog.message }}</div>
        <div class="confirm-actions">
          <button class="btn-cancel" @click="cancelConfirm">取消</button>
          <button class="btn-ok" @click="confirmAction">确认执行</button>
        </div>
      </div>
    </div>

    <!-- 预览弹框 -->
    <div v-if="previewEntry" class="modal-mask" @click.self="previewEntry = null">
      <div class="modal">
        <div class="modal-head">
          <div class="modal-title-group">
            <span class="modal-title">{{ previewEntry!.name }}</span>
            <span class="modal-badge">{{ previewEntry!.data.length }} 条</span>
          </div>
          <button class="close-btn" @click="previewEntry = null">×</button>
        </div>
        <div class="modal-body">
          <table class="preview-table">
            <thead>
              <tr>
                <th class="col-seq">#</th>
                <th v-for="(h, i) in previewEntry!.headers" :key="i">{{ h }}</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="(row, ri) in getPreviewRows()" :key="ri">
                <td class="col-seq">{{ (previewPage - 1) * previewPageSize + ri + 1 }}</td>
                <td v-for="(cell, ci) in row" :key="ci">{{ cell }}</td>
              </tr>
            </tbody>
          </table>
        </div>
        <div class="modal-foot">
          <span class="foot-info">
            第 {{ (previewPage - 1) * previewPageSize + 1 }}
            – {{ Math.min(previewPage * previewPageSize, previewEntry!.data.length) }} 条，
            共 {{ previewEntry!.data.length }} 条
          </span>
          <div class="pagination">
            <button :disabled="previewPage === 1" @click="previewPage--">上一页</button>
            <span>{{ previewPage }} / {{ previewTotalPages() }}</span>
            <button :disabled="previewPage >= previewTotalPages()" @click="previewPage++">下一页</button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.data-manager {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
  position: relative;
}

.toast {
  position: fixed;
  top: 20px; right: 20px;
  padding: 12px 20px;
  border-radius: 6px;
  font-size: 14px;
  z-index: 9999;
  box-shadow: 0 4px 16px rgba(0,0,0,0.15);
  max-width: 480px;
}
.toast-error   { background: #ff4d4f; color: #fff; }
.toast-success { background: #52c41a; color: #fff; }
.fade-enter-active, .fade-leave-active { transition: opacity 0.3s; }
.fade-enter-from, .fade-leave-to { opacity: 0; }

.top-bar {
  display: flex;
  align-items: center;
  padding: 0 24px;
  height: 52px;
  background: #fff;
  border-bottom: 1px solid #e8e8e8;
  flex-shrink: 0;
}
.page-title { font-size: 16px; font-weight: 600; color: #262626; }

.content {
  flex: 1;
  overflow-y: auto;
  padding: 28px 24px;
}
.status-center {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 200px;
}
.loading-text { color: #1677ff; font-size: 14px; }

.card-grid { display: flex; flex-wrap: wrap; gap: 20px; }

.card-scene {
  width: 280px;
  height: 190px;
  perspective: 1000px;
  position: relative;
}
.card-face {
  position: absolute;
  inset: 0;
  border-radius: 12px;
  backface-visibility: hidden;
  -webkit-backface-visibility: hidden;
  transition: transform 0.5s cubic-bezier(0.4, 0.2, 0.2, 1);
}

.card-front {
  background: #fff;
  border: 1px solid #e8e8e8;
  display: flex;
  flex-direction: column;
  padding: 20px;
  cursor: pointer;
  transform: rotateY(0deg);
  box-shadow: 0 2px 8px rgba(0,0,0,0.06);
}
.card-scene.flipped .card-front { transform: rotateY(-180deg); }
.card-scene:hover .card-front {
  border-color: #1677ff55;
  box-shadow: 0 6px 20px rgba(22,119,255,0.12);
}
.card-front-icon { font-size: 26px; margin-bottom: 8px; }
.card-front-body { flex: 1; }
.card-label { font-size: 12px; color: #8c8c8c; margin-bottom: 4px; letter-spacing: 0.5px; }
.card-count { margin-bottom: 4px; }
.count-num  { font-size: 30px; font-weight: 700; color: #1677ff; line-height: 1; }
.count-unit { font-size: 13px; color: #8c8c8c; }
.card-desc  { font-size: 12px; color: #bfbfbf; }
.card-flip-hint {
  font-size: 12px; color: #bfbfbf; text-align: right; margin-top: 6px; transition: color 0.2s;
}
.card-scene:hover .card-flip-hint { color: #1677ff; }

.card-back {
  background: #fff;
  border: 1px solid #e8e8e8;
  display: flex;
  flex-direction: column;
  align-items: stretch;
  padding: 16px;
  gap: 8px;
  transform: rotateY(180deg);
  box-shadow: 0 2px 8px rgba(0,0,0,0.06);
}
.card-scene.flipped .card-back { transform: rotateY(0deg); }

.back-title {
  font-size: 13px; font-weight: 600; color: #595959; text-align: center; margin-bottom: 2px;
}
.back-actions { display: flex; flex-direction: column; gap: 7px; flex: 1; }

.action-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border: none;
  border-radius: 6px;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
  width: 100%;
}
.btn-preview { background: #f5f5f5; color: #595959; border: 1px solid #e8e8e8; }
.btn-preview:hover { background: #e6f4ff; color: #1677ff; border-color: #91caff; }
.btn-danger  { background: #fff2f0; color: #ff4d4f; border: 1px solid #ffccc7; }
.btn-danger:hover:not(:disabled) { background: #ff4d4f; color: #fff; }
.btn-danger:disabled { opacity: 0.5; cursor: not-allowed; }
.action-icon { font-size: 13px; }

.modal-mask {
  position: fixed;
  inset: 0;
  background: rgba(0,0,0,0.45);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.confirm-dialog {
  background: #fff;
  border: 1px solid #e8e8e8;
  border-radius: 10px;
  padding: 28px 32px;
  width: 400px;
  box-shadow: 0 8px 32px rgba(0,0,0,0.12);
}
.confirm-title { font-size: 17px; font-weight: 600; color: #262626; margin-bottom: 12px; }
.confirm-msg   { font-size: 14px; color: #595959; line-height: 1.6; margin-bottom: 24px; }
.confirm-actions { display: flex; gap: 12px; justify-content: flex-end; }
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

.modal {
  background: #fff;
  border-radius: 10px;
  width: 92%;
  max-width: 1100px;
  max-height: 82vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 8px 32px rgba(0,0,0,0.12);
}
.modal-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid #f0f0f0;
  flex-shrink: 0;
}
.modal-title-group { display: flex; align-items: center; gap: 10px; }
.modal-title { font-size: 16px; font-weight: 600; color: #262626; }
.modal-badge {
  padding: 2px 10px; background: #e6f4ff; color: #1677ff;
  border-radius: 20px; font-size: 12px; border: 1px solid #91caff;
}
.close-btn {
  width: 30px; height: 30px; background: #f5f5f5; border: none;
  border-radius: 6px; color: #8c8c8c; font-size: 20px; cursor: pointer;
  line-height: 1; transition: all 0.2s;
}
.close-btn:hover { background: #ff4d4f; color: #fff; }
.modal-body { flex: 1; overflow: auto; }
.preview-table {
  width: max-content;
  min-width: 100%;
  border-collapse: collapse;
  font-size: 13px;
}
.preview-table th,
.preview-table td {
  padding: 8px 12px;
  border-bottom: 1px solid #f0f0f0;
  white-space: nowrap;
  text-align: left;
}
.preview-table thead th {
  background: #fafafa; color: #595959; font-weight: 600;
  position: sticky; top: 0; z-index: 10; border-bottom: 1px solid #e8e8e8;
}
.preview-table tbody td { color: #333; background: #fff; }
.preview-table tbody tr:hover td { background: #e6f4ff; }
.col-seq {
  width: 46px; min-width: 46px; text-align: center;
  background: #fafafa !important; color: #bfbfbf !important;
  position: sticky; left: 0; z-index: 5;
}
.modal-foot {
  display: flex; align-items: center; justify-content: space-between;
  padding: 12px 20px; border-top: 1px solid #f0f0f0; flex-shrink: 0;
}
.foot-info { font-size: 13px; color: #8c8c8c; }
.pagination { display: flex; align-items: center; gap: 12px; }
.pagination button {
  padding: 5px 14px; background: #fff; border: 1px solid #d9d9d9; border-radius: 4px;
  color: #595959; font-size: 13px; cursor: pointer; transition: all 0.2s;
}
.pagination button:disabled { opacity: 0.4; cursor: not-allowed; }
.pagination button:hover:not(:disabled) { border-color: #1677ff; color: #1677ff; }
.pagination span { color: #595959; font-size: 13px; min-width: 60px; text-align: center; }
</style>
