import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';

export type DetailRow = Record<string, string>;

export const DETAIL_COLS = [
  '序号', '产品名称', '规格', '特征', '数量', '单位', '单价',
  '金额', '下单人', '安装位置', '备注', '初始报价', '税率', '成本单价含税',
] as const;

export interface BasicInfo {
  项目名称: string;
  客户: string;
  供应商: string;
  销售日期: string;
  合同号: string;
}

export function isValidDate(val: string): boolean {
  return /^\d{4}-\d{2}-\d{2}$/.test(val.trim());
}

export function recalcRow(row: DetailRow) {
  const qty = parseFloat(row['数量'] ?? '');
  const price = parseFloat(row['单价'] ?? '');
  row['金额'] = (!isNaN(qty) && !isNaN(price))
    ? String(Math.round(qty * price * 100) / 100)
    : '';
}

export function buildRowData(basicInfo: BasicInfo, row: DetailRow): Record<string, string> {
  return {
    客户: basicInfo.客户,
    销售日期: basicInfo.销售日期,
    合同号: basicInfo.合同号,
    项目名称: basicInfo.项目名称,
    供应商: basicInfo.供应商,
    下单人: row['下单人'] ?? '',
    序号: row['序号'] ?? '',
    产品名称: row['产品名称'] ?? '',
    规格: row['规格'] ?? '',
    特征: row['特征'] ?? '',
    数量: row['数量'] ?? '',
    单位: row['单位'] ?? '',
    单价: row['单价'] ?? '',
    安装位置: row['安装位置'] ?? '',
    备注: row['备注'] ?? '',
    初始报价: row['初始报价'] ?? '',
    税率: row['税率'] ? String(parseFloat(row['税率']) / 100) : '',
    成本单价含税: row['成本单价含税'] ?? '',
    签收人: row['签收人'] ?? '',
    签收日期: row['签收日期'] ?? '',
    与客户对账时间: row['与客户对账时间'] ?? '',
    对账数量: row['对账数量'] ?? '',
    对账单价: row['对账单价'] ?? '',
    对账日期: row['对账日期'] ?? '',
    对账金额: row['对账金额'] ?? '',
    对账备注: row['对账备注'] ?? '',
  };
}

export function useContractForm(initialBasic: BasicInfo) {
  const basicInfo = ref<BasicInfo>({ ...initialBasic });
  const detailRows = ref<DetailRow[]>([]);
  const isSaving = ref(false);
  const toastMsg = ref('');

  const canSave = computed(() =>
    basicInfo.value.合同号.trim() !== '' &&
    basicInfo.value.客户.trim() !== '' &&
    (basicInfo.value.销售日期 === '' || isValidDate(basicInfo.value.销售日期)) &&
    detailRows.value.length > 0
  );

  function addRow() {
    const row: DetailRow = {};
    DETAIL_COLS.forEach(c => { row[c] = ''; });
    detailRows.value.push(row);
  }

  function removeRow(idx: number) {
    detailRows.value.splice(idx, 1);
  }

  async function saveRows(rowsToSave: DetailRow[], deletedIds: number[] = []) {
    // 删除被移除的行
    for (const id of deletedIds) {
      await invoke('delete_sales_row', { id });
    }
    // 保存/新增现有行
    for (const row of rowsToSave) {
      const rowData = buildRowData(basicInfo.value, row);
      const id = row['__id'] ? Number(row['__id']) : null;
      await invoke('save_sales_row', { id, rowData });
    }
  }

  return {
    basicInfo,
    detailRows,
    isSaving,
    toastMsg,
    canSave,
    addRow,
    removeRow,
    saveRows,
  };
}
