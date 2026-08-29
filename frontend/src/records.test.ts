import { describe, expect, it } from 'vitest';
import { SAMPLE_DOCUMENT } from './sample';
import { accountantCsv, hmrcHandoff, parseCsv, summarise } from './records';
import { currentUkQuarter, nextUkQuarter, quarterFromStart } from './quarters';

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
    const rows = parseCsv('date,description,amount,type\n2026-04-08,Lesson,45.00,income', '2026-04-06', '2026-07-05');
    expect(rows[0]).toMatchObject({ description: 'Lesson', amountPence: 4500, kind: 'income' });
  });

  it.each([
    ['impossible date', 'date,description,amount,type,category\n2026-02-30,Lesson,45.00,income,Sales', 'real calendar date'],
    ['out-of-quarter date', 'date,description,amount,type,category\n2026-07-06,Lesson,45.00,income,Sales', 'between 2026-04-06 and 2026-07-05'],
    ['zero value', 'date,description,amount,type,category\n2026-04-08,Lesson,0,income,Sales', 'between £0.01 and £1,000,000'],
    ['unknown category', 'date,description,amount,type,category\n2026-04-08,Lesson,45.00,income,Bananas', 'unknown category'],
  ])('rejects a CSV row with an %s', (_name, csv, message) => {
    expect(() => parseCsv(csv, '2026-04-06', '2026-07-05')).toThrow(message);
  });

  it('generates the current and subsequent standard UK quarters', () => {
    const current = currentUkQuarter(new Date('2026-08-29T12:00:00Z'));
    expect(current).toEqual(quarterFromStart('2026-07-06'));
    expect(nextUkQuarter(current)).toEqual(quarterFromStart('2026-10-06'));
    expect(nextUkQuarter(quarterFromStart('2026-10-06')!)).toEqual(quarterFromStart('2027-01-06'));
  });
});
