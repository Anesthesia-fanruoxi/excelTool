<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import {
  useContractForm, DETAIL_COLS, isValidDate, recalcRow,
  type DetailRow,
} from '@/composables/useContractForm';

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'saved'): void;
}>();

const {
  basicInfo, detailRows, isSaving, toastMsg, canSave, addRow, removeRow, saveRows,
} = useContractForm({ 项目名称: '', 客户: '', 供应商: '', 销售日期: '', 合同号: '' });

// ── 粘贴解析 ──────────────────────────────────────────────
const PASTE_COLS = ['序号', '产品名称', '规格', '特征', '数量', '单位', '单价', '_金额', '下单人', '安装位置', '备注'];
const pasteText = ref('');
const parseError = ref('');

function parseTSV(raw: string): string[][] {
  const result: string[][] = [];
  let row: string[] = [];
  let cell = '';
  let inQuote = false;
  let i = 0;
  const text = raw.replace(/\r\n/g, '\n').replace(/\r/g, '\n');
  while (i < text.length) {
    const ch = text[i];
    if (inQuote) {
      if (ch === '"') {
        if (text[i + 1] === '"') { cell += '"'; i += 2; continue; }
        inQuote = false; i++; continue;
      }
      cell += ch; i++; continue;
    }
    if (ch === '"') { inQuote = true; i++; continue; }
    if (ch === '\t') { row.push(cell.trim()); cell = ''; i++; continue; }
    if (ch === '\n') {
      row.push(cell.trim()); cell = '';
      if (row.some(c => c !== '')) result.push(row);
      row = []; i++; continue;
    }
    cell += ch; i++;
  }
  row.push(cell.trim());
  if (row.some(c => c !== '')) result.push(row);
  return result;
}

function buildDetailRow(cells: string[]): DetailRow {
  const row: DetailRow = {};
  PASTE_COLS.forEach((col, i) => {
    if (col === '_金额') return;
    row[col] = (cells[i] ?? '').trim();
  });
  recalcRow(row);
  return row;
}

function parsePaste() {
  parseError.value = '';
  const text = pasteText.value.trim();
  if (!text) { detailRows.value = []; return; }
  const lines = parseTSV(text);
  const rows = lines.map(cells => buildDetailRow(cells));
  if (rows.length === 0) {
    parseError.value = '未能解析出有效数据，请确认是从 Excel 直接复制的内容';
    return;
  }
  detailRows.value = rows;
}

function parseFromHtml(html: string): DetailRow[] {
  const parser = new DOMParser();
  const doc = parser.parseFromString(html, 'text/html');
  const rows: DetailRow[] = [];
  doc.querySelectorAll('tr').forEach(tr => {
    const cells = Array.from(tr.querySelectorAll('td, th')).map(td => (td.textContent ?? '').trim());
    if (cells.length === 0 || cells.every(c => c === '')) return;
    const row = buildDetailRow(cells);
    if (!row['产品名称'] && !row['数量']) return;
    rows.push(row);
  });
  return rows;
}

function onPaste(e: ClipboardEvent) {
  e.preventDefault();
  const html = e.clipboardData?.getData('text/html') ?? '';
  if (html) {
    const rows = parseFromHtml(html);
    if (rows.length > 0) {
      detailRows.value = rows;
      pasteText.value = `[已从 HTML 解析 ${rows.length} 行]`;
      parseError.value = '';
      return;
    }
  }
  const text = e.clipboardData?.getData('text/plain') ?? '';
  pasteText.value = text;
  parsePaste();
}

// ── 保存 ──────────────────────────────────────────────────
async function doSave() {
  if (!canSave.value) return;
  isSaving.value = true;
  try {
    await saveRows(detailRows.value);
    toastMsg.value = `保存成功，共 ${detailRows.value.length} 条明细`;
    setTimeout(() => { emit('saved'); emit('close'); }, 1200);
  } catch (e) {
    toastMsg.value = `保存失败: ${e}`;
  } finally {
    isSaving.value = false;
  }
}

// ── 关闭确认 ──────────────────────────────────────────────
const showCloseConfirm = ref(false);

function hasData(): boolean {
  return Object.values(basicInfo.value).some(v => v.trim() !== '') ||
    detailRows.value.length > 0 || pasteText.value.trim() !== '';
}

function tryClose() {
  if (hasData()) { showCloseConfirm.value = true; } else { emit('close'); }
}

function confirmClose() {
  showCloseConfirm.value = false;
  emit('close');
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    if (showCloseConfirm.value) { showCloseConfirm.value = false; return; }
    tryClose();
  }
}
onMounted(() => window.addEventListener('keydown', onKeydown));
onUnmounted(() => window.removeEventListener('keydown', onKeydown));
</script>

<template>
  <div class="modal-mask">
    <div class="modal">
      <!-- 头部 -->
      <div class="modal-head">
        <span class="modal-title">新增合同</span>
        <button class="close-btn" @click="tryClose">×</button>
      </div>

      <div class="modal-body">
        <!-- Toast -->
        <transition name="fade">
          <div v-if="toastMsg" :class="['inline-toast', toastMsg.includes('失败') ? 'toast-error' : 'toast-success']">
            {{ toastMsg }}
          </div>
        </transition>

        <!-- 基本信息 -->
        <div class="section">
          <div class="section-title">基本信息</div>
          <div class="basic-grid">
            <div v-for="field in Object.keys(basicInfo)" :key="field" class="form-item">
              <label>{{ field }}<span v-if="['合同号','客户'].includes(field)" class="required">*</span></label>
              <input
                v-model="basicInfo[field as keyof typeof basicInfo]"
                :placeholder="field === '销售日期' ? '2026-04-22' : field"
                :class="{ 'input-warn': field === '销售日期' && basicInfo.销售日期 && !isValidDate(basicInfo.销售日期) }"
              />
              <span v-if="field === '销售日期' && basicInfo.销售日期 && !isValidDate(basicInfo.销售日期)" class="field-warn">
                格式应为 2026-04-22
              </span>
            </div>
          </div>
        </div>

        <!-- 产品明细 -->
        <div class="section">
          <div class="section-title">
            产品明细
            <span class="section-hint">在下方文本框粘贴 Excel 数据（Ctrl+V），自动解析</span>
          </div>

          <!-- 粘贴区 -->
          <textarea
            v-model="pasteText"
            class="paste-area"
            placeholder="从 Excel 复制数据后，点击此处粘贴（Ctrl+V）&#10;列顺序：序号、产品名称、规格、特征、数量、单位、单价、金额、下单人、安装位置、备注"
            rows="3"
            @paste="onPaste"
          />
          <div v-if="parseError" class="parse-error">{{ parseError }}</div>

          <!-- 预览表格 -->
          <div v-if="detailRows.length > 0" class="preview-wrap">
            <div class="preview-toolbar">
              <span class="row-count">已解析 <b>{{ detailRows.length }}</b> 行</span>
              <button class="btn-add-row" @click="addRow">+ 手动添加行</button>
            </div>
            <div class="table-scroll">
              <table class="preview-table">
                <thead>
                  <tr>
                    <th class="col-op"></th>
                    <th v-for="col in DETAIL_COLS" :key="col">{{ col }}</th>
                  </tr>
                </thead>
                <tbody>
                  <tr v-for="(row, idx) in detailRows" :key="idx">
                    <td class="col-op">
                      <button class="btn-del-row" @click="removeRow(idx)">×</button>
                    </td>
                    <td v-for="col in DETAIL_COLS" :key="col">
                      <div v-if="col === '税率'" class="input-wrap">
                        <input
                          v-model="row[col]"
                          class="cell-input"
                          placeholder="如 13"
                          type="number"
                        />
                        <span class="cell-suffix">%</span>
                      </div>
                      <input
                        v-else
                        v-model="row[col]"
                        class="cell-input"
                        :class="{ 'cell-computed': col === '金额' }"
                        :readonly="col === '金额'"
                        @input="col !== '金额' && recalcRow(row)"
                      />
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>

          <div v-else class="empty-detail">
            <span>暂无明细，请粘贴 Excel 数据或手动添加</span>
            <button class="btn-add-row" @click="addRow">+ 手动添加行</button>
          </div>
        </div>
      </div>

      <!-- 底部 -->
      <div class="modal-foot">
        <span class="foot-hint" v-if="!canSave && detailRows.length === 0">请填写合同号、客户并添加产品明细</span>
        <button class="btn-cancel" @click="tryClose">取消</button>
        <button class="btn-save" :disabled="!canSave || isSaving" @click="doSave">
          {{ isSaving ? '保存中...' : `保存 (${detailRows.length} 条明细)` }}
        </button>
      </div>
    </div>
    <!-- 关闭确认 -->
    <transition name="fade">
      <div v-if="showCloseConfirm" class="confirm-mask" @click.self="showCloseConfirm = false">
        <div class="confirm-dialog">
          <div class="confirm-title">⚠️ 确认关闭</div>
          <div class="confirm-msg">当前已填写的数据将会丢失，确认关闭？</div>
          <div class="confirm-actions">
            <button class="btn-cancel" @click="showCloseConfirm = false">继续编辑</button>
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
  width: 1000px; max-width: 96vw; max-height: 90vh;
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

.modal-body { flex: 1; overflow-y: auto; padding: 16px 20px; display: flex; flex-direction: column; gap: 20px; }

/* Toast */
.inline-toast { padding: 8px 14px; border-radius: 6px; font-size: 13px; }
.toast-success { background: #f6ffed; color: #52c41a; border: 1px solid #b7eb8f; }
.toast-error   { background: #fff2f0; color: #ff4d4f; border: 1px solid #ffccc7; }
.fade-enter-active, .fade-leave-active { transition: opacity 0.3s; }
.fade-enter-from, .fade-leave-to { opacity: 0; }

/* 分区 */
.section { display: flex; flex-direction: column; gap: 10px; }
.section-title {
  font-size: 13px; font-weight: 600; color: #1677ff;
  padding-bottom: 8px; border-bottom: 1px solid #f0f0f0;
  display: flex; align-items: center; gap: 10px;
}
.section-hint { font-size: 12px; color: #8c8c8c; font-weight: normal; }

/* 基本信息 */
.basic-grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 10px 16px; }
.form-item { display: flex; flex-direction: column; gap: 4px; }
.form-item label { font-size: 12px; color: #8c8c8c; }
.required { color: #ff4d4f; margin-left: 2px; }
.form-item input {
  padding: 7px 10px; background: #fff; border: 1px solid #d9d9d9;
  border-radius: 4px; color: #262626; font-size: 13px;
}
.form-item input:focus { outline: none; border-color: #1677ff; box-shadow: 0 0 0 2px rgba(22,119,255,0.1); }
.input-warn { border-color: #ff4d4f !important; }
.field-warn { font-size: 11px; color: #ff4d4f; margin-top: 2px; }

/* 粘贴区 */
.paste-area {
  width: 100%; padding: 10px 12px; border: 1px dashed #d9d9d9; border-radius: 6px;
  font-size: 12px; color: #595959; resize: vertical; font-family: monospace;
  background: #fafafa; transition: border-color 0.2s; box-sizing: border-box;
}
.paste-area:focus { outline: none; border-color: #1677ff; background: #fff; }
.parse-error { font-size: 12px; color: #ff4d4f; }

/* 预览表格 */
.preview-wrap { display: flex; flex-direction: column; gap: 8px; }
.preview-toolbar { display: flex; align-items: center; justify-content: space-between; }
.row-count { font-size: 12px; color: #8c8c8c; }
.row-count b { color: #1677ff; }
.btn-add-row {
  padding: 4px 12px; background: #fff; border: 1px solid #d9d9d9;
  border-radius: 4px; color: #595959; font-size: 12px; cursor: pointer; transition: all 0.2s;
}
.btn-add-row:hover { border-color: #1677ff; color: #1677ff; }

.table-scroll { overflow-x: auto; border: 1px solid #f0f0f0; border-radius: 6px; }
.preview-table { border-collapse: collapse; font-size: 12px; min-width: 100%; }
.preview-table th, .preview-table td {
  padding: 4px 6px; border-bottom: 1px solid #f0f0f0; border-right: 1px solid #f0f0f0;
  white-space: nowrap; text-align: left;
}
.preview-table th:last-child, .preview-table td:last-child { border-right: none; }
.preview-table thead th { background: #fafafa; color: #595959; font-weight: 600; position: sticky; top: 0; }
.col-op { width: 32px; text-align: center; }
.btn-del-row {
  width: 20px; height: 20px; border: none; background: transparent;
  color: #bfbfbf; font-size: 14px; cursor: pointer; line-height: 1; transition: color 0.2s;
}
.btn-del-row:hover { color: #ff4d4f; }
.cell-input {
  width: 100%; min-width: 60px; padding: 3px 6px; border: 1px solid transparent;
  border-radius: 3px; font-size: 12px; color: #262626; background: transparent;
  box-sizing: border-box;
}
.cell-input:focus { outline: none; border-color: #1677ff; background: #fff; }
.cell-computed { color: #1677ff !important; cursor: not-allowed; }
.input-wrap { position: relative; display: flex; align-items: center; }
.input-wrap .cell-input { padding-right: 20px; }
.cell-suffix { position: absolute; right: 4px; font-size: 11px; color: #8c8c8c; pointer-events: none; }

.empty-detail {
  display: flex; align-items: center; justify-content: center; gap: 12px;
  padding: 24px; border: 1px dashed #e8e8e8; border-radius: 6px;
  color: #bfbfbf; font-size: 13px;
}

/* 底部 */
.modal-foot {
  display: flex; align-items: center; justify-content: flex-end; gap: 10px;
  padding: 14px 20px; border-top: 1px solid #f0f0f0; flex-shrink: 0;
}
.foot-hint { font-size: 12px; color: #bfbfbf; margin-right: auto; }
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

/* 关闭确认 */
.confirm-mask {
  position: fixed; inset: 0; background: rgba(0,0,0,0.6);
  display: flex; align-items: center; justify-content: center; z-index: 10;
}
.confirm-dialog {
  background: #fff; border-radius: 10px; padding: 24px 28px; width: 380px;
  box-shadow: 0 8px 32px rgba(0,0,0,0.2);
}
.confirm-title { font-size: 16px; font-weight: 600; color: #262626; margin-bottom: 12px; }
.confirm-msg { font-size: 14px; color: #595959; line-height: 1.6; margin-bottom: 20px; }
.confirm-actions { display: flex; gap: 10px; justify-content: flex-end; }
.btn-ok {
  padding: 7px 18px; background: #ff4d4f; border: none; border-radius: 6px;
  color: #fff; font-size: 14px; cursor: pointer; transition: background 0.2s;
}
.btn-ok:hover { background: #ff7875; }
</style>
