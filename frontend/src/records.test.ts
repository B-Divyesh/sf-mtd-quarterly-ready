import { describe, expect, it } from 'vitest';
import { SAMPLE_DOCUMENT } from './sample';
import { accountantCsv, hmrcHandoff, parseCsv, summarise } from './records';

describe('quarter records', () => {
  it('calculates the sample quarter', () => {
    expect(summarise(SAMPLE_DOCUMENT)).toEqual({ incomePence: 26000, expensePence: 15583, netPence: 10417, unresolved: 1, missingReceipts: 1 });
  });
  it('puts every transaction in the accountant pack', () => {
    const csv = accountantCsv(SAMPLE_DOCUMENT);
    expect(csv).toContain('GCSE maths lesson');
    expect(csv.split('\r\n')).toHaveLength(19);
  });
  it('builds an MTD handoff with period totals', () => {
    const handoff = hmrcHandoff(SAMPLE_DOCUMENT) as Record<string, unknown>;
    expect(handoff.format).toBe('quarterly-ready-mtd-itsa-handoff-v1');
    expect(handoff.periodStartDate).toBe('2026-04-06');
  });
  it('imports bank-style CSV rows', () => {
    const rows = parseCsv('date,description,amount,type\n2026-04-08,Lesson,45.00,income');
    expect(rows[0]).toMatchObject({ description: 'Lesson', amountPence: 4500, kind: 'income' });
  });
});
