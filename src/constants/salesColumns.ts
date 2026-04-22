/** 销售表标准列（与 Excel 模板表头完全一致） */
export const SALES_COLUMNS = [
  '客户',
  '销售日期',
  '合同号',
  '送货单号',
  '项目名称',
  '收货地址',
  '序号',
  '产品名称',
  '规格',
  '特征',
  '数量',
  '单位',
  '单价',
  '金额',
  '下单人',
  '安装位置',
  '备注',
  '所属年份',
  '签收人',
  '签收日期',
  '与客户对账时间',
  '状态列',
  '供应商',
  '初始报价',
  '税率',
  '成本单价含税',
  '应付金额',
  '对账数量',
  '对账单价',
  '对账日期',
  '对账金额',
  '对账备注',
  '利润',
] as const;

/** 计算列：不允许手动编辑，由其他字段自动计算 */
export const COMPUTED_COLUMNS = new Set([
  '金额',
  '应付金额',
  '利润',
  '状态列',
]);

/**
 * 计算一行中的派生字段
 * 调用时机：表单任意字段变化后
 */
export function computeRow(row: SalesRow): SalesRow {
  const result = { ...row };

  // 清理数值：去除逗号、空格等非数字字符
  function toNum(val: string): number {
    return parseFloat(val.replace(/,/g, '').trim());
  }

  // 金额 = 单价 × 数量
  const unitPrice = toNum(result['单价'] ?? '');
  const qty = toNum(result['数量'] ?? '');
  result['金额'] =
    !result['单价']?.trim() || !result['数量']?.trim()
      ? ''
      : isNaN(unitPrice) || isNaN(qty)
      ? ''
      : String(Math.round(unitPrice * qty * 100) / 100);

  // 应付金额 = 成本单价含税 × 数量
  const costPrice = toNum(result['成本单价含税'] ?? '');
  result['应付金额'] =
    !result['成本单价含税']?.trim() || !result['数量']?.trim()
      ? ''
      : isNaN(costPrice) || isNaN(qty)
      ? ''
      : String(Math.round(costPrice * qty * 100) / 100);

  // 利润 = 金额 - 应付金额
  const amount = toNum(result['金额'] ?? '');
  const payable = toNum(result['应付金额'] ?? '');
  result['利润'] =
    !result['应付金额']?.trim() || !result['金额']?.trim()
      ? ''
      : isNaN(amount) || isNaN(payable)
      ? ''
      : String(Math.round((amount - payable) * 100) / 100);

  // 状态列：基于数量（N列）是否有值来判断是否有效行
  const hasData = !!result['数量']?.trim();
  const signPerson = result['签收人']?.trim() ?? '';
  const signDate = result['签收日期']?.trim() ?? '';
  const reconcileTime = result['与客户对账时间']?.trim() ?? '';

  if (!hasData) {
    result['状态列'] = '';
  } else if (reconcileTime !== '') {
    result['状态列'] = '已对账';
  } else if (signPerson !== '' && signDate !== '') {
    result['状态列'] = '待对账';
  } else if (signPerson !== '' && signDate === '') {
    result['状态列'] = '回签不完整';
  } else {
    result['状态列'] = '等回签';
  }

  return result;
}

/** 表单分组（新增/编辑弹框用） */
export const SALES_FORM_GROUPS = [
  {
    label: '基本信息',
    fields: ['客户', '销售日期', '合同号', '送货单号', '项目名称', '收货地址', '下单人', '所属年份', '状态列'],
  },
  {
    label: '产品明细',
    fields: ['序号', '产品名称', '规格', '特征', '数量', '单位', '单价', '金额', '安装位置', '备注'],
  },
  {
    label: '签收信息',
    fields: ['签收人', '签收日期', '与客户对账时间'],
  },
  {
    label: '对账 / 成本',
    fields: ['供应商', '初始报价', '税率', '成本单价含税', '应付金额', '对账数量', '对账单价', '对账日期', '对账金额', '对账备注', '利润'],
  },
] as const;

/** 列表页展示列（基本信息，不含产品明细） */
export const LIST_COLUMNS = [
  '客户',
  '销售日期',
  '合同号',
  '项目名称',
  '数量',
  '单价',
  '金额',
  '签收日期',
  '状态列',
  '利润',
] as const;

/** 产品明细列（默认在列表中隐藏） */
export const DETAIL_COLUMNS = new Set([
  '序号',
  '产品名称',
  '规格',
  '特征',
  '单位',
  '安装位置',
  '备注',
]);

export type SalesRow = Record<string, string>;
