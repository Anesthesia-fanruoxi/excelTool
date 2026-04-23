// ── 报价表 ────────────────────────────────────────────────

/** 数据库字段 → 前端显示名 */
export const QUOTE_FIELD_LABELS: Record<string, string> = {
  goods_name: '货物名称',
  spec:       '规格型号',
  unit:       '单位',
  supplier:   '供应商',
  tax_rate:   '税率',
  cost_price: '成本单价（含税）',
  date:       '日期',
  uuid:       'UUID',
};

/** 报价明细列表展示列（按顺序） */
export const QUOTE_LIST_FIELDS = [
  'goods_name',
  'spec',
  'unit',
  'supplier',
  'tax_rate',
  'cost_price',
  'date',
] as const;

export type QuoteField = typeof QUOTE_LIST_FIELDS[number];

/** 报价明细可搜索字段 */
export const QUOTE_FILTER_FIELDS: { field: string; label: string }[] = [
  { field: 'goods_name', label: '货物名称' },
  { field: 'spec',       label: '规格型号' },
  { field: 'supplier',   label: '供应商' },
];

// ── 销售表 ────────────────────────────────────────────────

/** 数据库字段 → 前端显示名 */
export const SALES_FIELD_LABELS: Record<string, string> = {
  contract_no:  '合同号',
  customer:     '客户',
  sale_date:    '销售日期',
  project_name: '项目名称',
  product_name: '产品名称',
  spec:         '规格',
  quantity:     '数量',
  unit:         '单位',
  unit_price:   '单价',
  supplier:     '供应商',
  remark:       '备注',
  item_uuid:    '关联UUID',
};

/** 销售明细列表展示列 */
export const SALES_LIST_FIELDS = [
  'contract_no',
  'customer',
  'sale_date',
  'project_name',
  'product_name',
  'spec',
  'quantity',
  'unit',
  'unit_price',
  'supplier',
  'remark',
] as const;

export type SalesField = typeof SALES_LIST_FIELDS[number];

/** 销售明细可搜索字段 */
export const SALES_FILTER_FIELDS: { field: string; label: string }[] = [
  { field: 'contract_no',  label: '合同号' },
  { field: 'customer',     label: '客户' },
  { field: 'product_name', label: '产品名称' },
  { field: 'spec',         label: '规格' },
  { field: 'supplier',     label: '供应商' },
];

// ── 合同管理 ──────────────────────────────────────────────

/** 合同管理前端显示名 */
export const CONTRACT_FIELD_LABELS: Record<string, string> = {
  contract_no:        '合同号',
  customer:           '客户',
  sale_date:          '销售日期',
  project_name:       '项目名称',
  product_count:      '产品数',
  total_sale_amount:  '销售金额',
  total_cost_amount:  '成本金额',
  total_profit:       '利润',
  unlinked_count:     '未关联数',
};

// ── 通用类型 ──────────────────────────────────────────────

export interface QuoteItem {
  id: number;
  uuid: string;
  goods_name: string;
  spec: string;
  unit: string;
  supplier: string;
  tax_rate: string;
  cost_price: number;
  date: string;
}

export interface SalesItem {
  id: number;
  contract_no: string;
  customer: string;
  sale_date: string;
  project_name: string;
  product_name: string;
  spec: string;
  quantity: number;
  unit: string;
  unit_price: number;
  supplier: string;
  remark: string;
  item_uuid: string | null;
}

export interface ContractRow {
  contract_no: string;
  customer: string;
  sale_date: string;
  project_name: string;
  product_count: number;
  total_sale_amount: number;
  total_cost_amount: number;
  total_profit: number;
  unlinked_count: number;
}

export interface ContractDetailRow {
  id: number;
  product_name: string;
  spec: string;
  quantity: number;
  unit: string;
  unit_price: number;
  supplier: string;
  remark: string;
  item_uuid: string | null;
  goods_name: string | null;
  cost_price: number | null;
  tax_rate: string | null;
  sale_amount: number;
  cost_amount: number | null;
  profit: number | null;
}
