import type { QuarterDocument } from './types';
import { currentUkQuarter, type UkQuarter } from './quarters';

export const SAMPLE_DOCUMENT: QuarterDocument = {
  schemaVersion: 1,
  businessName: 'Maya Patel Tutoring',
  quarterLabel: '6 April to 5 July 2026',
  quarterStart: '2026-04-06',
  quarterEnd: '2026-07-05',
  figuresReviewed: false,
  packDownloaded: false,
  markedReady: false,
  updatedAt: '2026-06-28T10:30:00.000Z',
  transactions: [
    { id: 'sample-01', date: '2026-04-09', description: 'GCSE maths lesson — A. Lewis', amountPence: 4500, kind: 'income', category: 'Sales' },
    { id: 'sample-02', date: '2026-04-18', description: 'Printer paper and ink', amountPence: 3899, kind: 'expense', category: 'Office costs', receiptName: 'stationery-18-apr.pdf' },
    { id: 'sample-03', date: '2026-04-24', description: 'A-level physics lesson — T. Shah', amountPence: 6000, kind: 'income', category: 'Sales' },
    { id: 'sample-04', date: '2026-05-03', description: 'Train to student home', amountPence: 1260, kind: 'expense', category: 'Travel', receiptName: 'rail-ticket-03-may.jpg' },
    { id: 'sample-05', date: '2026-05-10', description: 'Exam practice books', amountPence: 2425, kind: 'expense', category: 'Office costs', receiptName: 'books-10-may.pdf' },
    { id: 'sample-06', date: '2026-05-19', description: 'GCSE science lesson — R. Jones', amountPence: 4500, kind: 'income', category: 'Sales' },
    { id: 'sample-07', date: '2026-06-02', description: 'Professional teaching membership', amountPence: 6800, kind: 'expense', category: 'Professional fees', receiptName: 'membership-02-jun.pdf' },
    { id: 'sample-08', date: '2026-06-11', description: 'Bank transfer from J. Clarke', amountPence: 5000, kind: 'income', category: '' },
    { id: 'sample-09', date: '2026-06-17', description: 'Whiteboard markers', amountPence: 1199, kind: 'expense', category: 'Office costs' },
    { id: 'sample-10', date: '2026-06-26', description: 'A-level maths lesson — K. Brown', amountPence: 6000, kind: 'income', category: 'Sales' }
  ]
};

export function emptyDocument(period: UkQuarter = currentUkQuarter()): QuarterDocument {
  return {
    schemaVersion: 1,
    businessName: '',
    quarterLabel: period.label,
    quarterStart: period.start,
    quarterEnd: period.end,
    transactions: [],
    figuresReviewed: false,
    packDownloaded: false,
    markedReady: false,
    updatedAt: new Date().toISOString()
  };
}
