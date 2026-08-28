export type Category = 'Sales' | 'Rent and rates' | 'Travel' | 'Office costs' | 'Professional fees' | 'Repairs' | 'Other' | '';

export interface Transaction {
  id: string;
  date: string;
  description: string;
  amountPence: number;
  kind: 'income' | 'expense';
  category: Category;
  receiptName?: string;
  receiptData?: string;
  note?: string;
}

export interface QuarterDocument {
  schemaVersion: 1;
  businessName: string;
  quarterLabel: string;
  quarterStart: string;
  quarterEnd: string;
  transactions: Transaction[];
  figuresReviewed: boolean;
  packDownloaded: boolean;
  markedReady: boolean;
  updatedAt: string;
}

export interface Summary {
  incomePence: number;
  expensePence: number;
  netPence: number;
  unresolved: number;
  missingReceipts: number;
}
