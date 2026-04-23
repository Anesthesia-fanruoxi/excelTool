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
  '产品名称',
  '数量',
  '单价',
  '金额',
  '签收日期',
  '状态列',
  '利润',
] as const;

/** 列表页支持过滤的列 */
export const FILTER_COLUMNS = [
  '客户',
  '销售日期',
  '合同号',
  '项目名称',
  '产品名称',
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

// ── 报价表 ────────────────────────────────────────────────

/** 报价表标准列（与 Excel 模板表头完全一致） */
export const QUOTE_COLUMNS = [
  '报价序号',
  '区域',
  '日期',
  '客户的合同号',
  '序号',
  '货物名称',
  '规格型号',
  '单位',
  '数量',
  '备注',
  '供应商',
  '列1',
  '税率',
  '成本单价（含税）',
  '金额',
  '销售单价(含税)',
  '金额2',
  '利润',
  '(成本➗销售价)',
  '最后成交单价',
  '金额3',
  '单价差异',
] as const;

/** 报价表计算列（由其他字段自动计算，不从 Excel 导入） */
export const QUOTE_COMPUTED_COLUMNS = new Set([
  '成本单价（含税）',
  '金额',
  '金额2',
  '利润',
  '(成本➗销售价)',
  '金额3',
  '单价差异',
]);

/**
 * 计算报价表一行的派生字段
 * 成本单价（含税）= 税率>12% ? 列1 : 列1/0.87
 * 金额            = 成本单价（含税）× 数量
 * 金额2           = 销售单价(含税) × 数量
 * 利润            = 金额2 - 金额
 * (成本➗销售价)  = 金额 / 金额2
 * 金额3           = 最后成交单价 × 数量
 * 单价差异        = 销售单价(含税) - 最后成交单价（>0 才显示）
 */
export function computeQuoteRow(row: Record<string, string>): Record<string, string> {
  const result = { ...row };

  function toNum(val: string | undefined): number {
    if (!val?.trim()) return NaN;
    return parseFloat(val.replace(/,/g, '').trim());
  }
  function fmt(n: number): string {
    return isNaN(n) ? '' : String(Math.round(n * 100) / 100);
  }

  const col1 = toNum(result['列1']);
  const taxRateRaw = result['税率']?.replace(/%/g, '').trim() ?? '';
  const taxRate = parseFloat(taxRateRaw);
  const qty = toNum(result['数量']);
  const salePrice = toNum(result['销售单价(含税)']);
  const finalPrice = toNum(result['最后成交单价']);

  // 成本单价（含税）
  let costPrice = NaN;
  if (!isNaN(col1)) {
    const rate = isNaN(taxRate) ? 0 : taxRate;
    costPrice = rate > 12 ? col1 : col1 / 0.87;
    result['成本单价（含税）'] = fmt(costPrice);
  } else {
    result['成本单价（含税）'] = '';
  }

  // 金额 = 成本单价（含税）× 数量
  const amount = isNaN(costPrice) || isNaN(qty) ? NaN : costPrice * qty;
  result['金额'] = fmt(amount);

  // 金额2 = 销售单价(含税) × 数量
  const amount2 = isNaN(salePrice) || isNaN(qty) ? NaN : salePrice * qty;
  result['金额2'] = fmt(amount2);

  // 利润 = 金额2 - 金额
  result['利润'] = isNaN(amount) || isNaN(amount2) ? '' : fmt(amount2 - amount);

  // (成本➗销售价) = 金额 / 金额2
  result['(成本➗销售价)'] = isNaN(amount) || isNaN(amount2) || amount2 === 0 ? '' : fmt(amount / amount2);

  // 金额3 = 最后成交单价 × 数量
  result['金额3'] = isNaN(finalPrice) || isNaN(qty) ? '' : fmt(finalPrice * qty);

  // 单价差异 = 销售单价(含税) - 最后成交单价，>0 才显示
  if (!isNaN(salePrice) && !isNaN(finalPrice)) {
    const diff = salePrice - finalPrice;
    result['单价差异'] = diff > 0 ? fmt(diff) : '';
  } else {
    result['单价差异'] = '';
  }

  return result;
}

export type QuoteRow = Record<string, string>;
