import type { QuarterDocument, Summary, Transaction } from './types';

export function summarise(document: QuarterDocument): Summary {
  const incomePence = document.transactions.filter(t => t.kind === 'income').reduce((sum, t) => sum + t.amountPence, 0);
  const expensePence = document.transactions.filter(t => t.kind === 'expense').reduce((sum, t) => sum + t.amountPence, 0);
  return {
    incomePence,
    expensePence,
    netPence: incomePence - expensePence,
    unresolved: document.transactions.filter(t => !t.category).length,
    missingReceipts: document.transactions.filter(t => t.kind === 'expense' && !t.receiptName).length
  };
}

export function pounds(pence: number): string {
  return new Intl.NumberFormat('en-GB', { style: 'currency', currency: 'GBP' }).format(pence / 100);
}

function quoteCsv(value: string | number): string {
  const text = String(value);
  return /[",\n]/.test(text) ? `"${text.replaceAll('"', '""')}"` : text;
}

export function accountantCsv(document: QuarterDocument): string {
  const summary = summarise(document);
  const lines = [
    ['Quarterly Ready accountant pack'],
    ['Business', document.businessName || 'Not entered'],
    ['Quarter', document.quarterLabel],
    ['Income', (summary.incomePence / 100).toFixed(2)],
    ['Expenses', (summary.expensePence / 100).toFixed(2)],
    ['Net', (summary.netPence / 100).toFixed(2)],
    ['Unresolved transactions', summary.unresolved],
    [],
    ['Date', 'Description', 'Type', 'Amount GBP', 'Category', 'Receipt', 'Note'],
    ...document.transactions.map(t => [t.date, t.description, t.kind, (t.amountPence / 100).toFixed(2), t.category || 'Needs a category', t.receiptName || '', t.note || ''])
  ];
  return lines.map(row => row.map(quoteCsv).join(',')).join('\r\n');
}

export function hmrcHandoff(document: QuarterDocument): object {
  const income = document.transactions.filter(t => t.kind === 'income').reduce((sum, t) => sum + t.amountPence, 0) / 100;
  const expenses = document.transactions.filter(t => t.kind === 'expense').reduce<Record<string, number>>((out, t) => {
    const key = (t.category || 'unresolved').toLowerCase().replaceAll(' ', '_');
    out[key] = Number(((out[key] || 0) + t.amountPence / 100).toFixed(2));
    return out;
  }, {});
  return {
    format: 'quarterly-ready-mtd-itsa-handoff-v1',
    periodStartDate: document.quarterStart,
    periodEndDate: document.quarterEnd,
    periodIncome: { turnover: Number(income.toFixed(2)) },
    periodExpenses: expenses,
    reviewedByUser: document.figuresReviewed,
    generatedAt: new Date().toISOString(),
    note: 'Review these figures in HMRC-recognised software before submission.'
  };
}

export function parseCsv(text: string): Omit<Transaction, 'id'>[] {
  const rows: string[][] = [];
  let row: string[] = [], cell = '', quoted = false;
  for (let i = 0; i < text.length; i++) {
    const char = text[i];
    if (char === '"' && quoted && text[i + 1] === '"') { cell += '"'; i++; }
    else if (char === '"') quoted = !quoted;
    else if (char === ',' && !quoted) { row.push(cell.trim()); cell = ''; }
    else if ((char === '\n' || char === '\r') && !quoted) {
      if (char === '\r' && text[i + 1] === '\n') i++;
      row.push(cell.trim()); if (row.some(Boolean)) rows.push(row); row = []; cell = '';
    } else cell += char;
  }
  row.push(cell.trim()); if (row.some(Boolean)) rows.push(row);
  if (rows.length < 2) throw new Error('The CSV needs a header row and at least one transaction.');
  const headers = rows[0].map(h => h.toLowerCase());
  const index = (names: string[]) => headers.findIndex(h => names.includes(h));
  const dateIndex = index(['date']), descriptionIndex = index(['description', 'details']), amountIndex = index(['amount', 'amount gbp']);
  const typeIndex = index(['type', 'kind']), categoryIndex = index(['category']);
  if ([dateIndex, descriptionIndex, amountIndex].some(i => i < 0)) throw new Error('The CSV needs date, description and amount columns.');
  return rows.slice(1).map((values, offset) => {
    const amount = Number(values[amountIndex]?.replace(/[£,]/g, ''));
    if (!/^\d{4}-\d{2}-\d{2}$/.test(values[dateIndex] || '') || !values[descriptionIndex] || !Number.isFinite(amount)) {
      throw new Error(`Row ${offset + 2} needs a valid date, description and amount.`);
    }
    const explicitType = values[typeIndex]?.toLowerCase();
    const kind = explicitType === 'expense' || amount < 0 ? 'expense' : 'income';
    return {
      date: values[dateIndex], description: values[descriptionIndex], amountPence: Math.round(Math.abs(amount) * 100), kind,
      category: (values[categoryIndex] || '') as Transaction['category']
    };
  });
}
