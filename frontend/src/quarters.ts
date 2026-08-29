import type { QuarterDocument } from './types';

export interface UkQuarter {
  start: string;
  end: string;
  label: string;
  shortLabel: string;
}

function iso(year: number, month: number, day: number): string {
  return `${year}-${String(month).padStart(2, '0')}-${String(day).padStart(2, '0')}`;
}

function taxYear(year: number): string {
  return `${year}–${String((year + 1) % 100).padStart(2, '0')}`;
}

export function quarterFromStart(start: string): UkQuarter | null {
  const match = /^(\d{4})-(01|04|07|10)-06$/.exec(start);
  if (!match) return null;
  const year = Number(match[1]);
  const month = Number(match[2]);
  if (month === 4) return { start, end: iso(year, 7, 5), label: `6 April to 5 July ${year}`, shortLabel: `Quarter 1 · ${taxYear(year)}` };
  if (month === 7) return { start, end: iso(year, 10, 5), label: `6 July to 5 October ${year}`, shortLabel: `Quarter 2 · ${taxYear(year)}` };
  if (month === 10) return { start, end: iso(year + 1, 1, 5), label: `6 October ${year} to 5 January ${year + 1}`, shortLabel: `Quarter 3 · ${taxYear(year)}` };
  return { start, end: iso(year, 4, 5), label: `6 January to 5 April ${year}`, shortLabel: `Quarter 4 · ${taxYear(year - 1)}` };
}

export function currentUkQuarter(now = new Date()): UkQuarter {
  const date = iso(now.getUTCFullYear(), now.getUTCMonth() + 1, now.getUTCDate());
  const year = now.getUTCFullYear();
  const starts = [iso(year - 1, 10, 6), iso(year, 1, 6), iso(year, 4, 6), iso(year, 7, 6), iso(year, 10, 6)];
  const start = starts.filter(candidate => candidate <= date).at(-1)!;
  return quarterFromStart(start)!;
}

export function nextUkQuarter(period: Pick<UkQuarter, 'start'>): UkQuarter {
  const year = Number(period.start.slice(0, 4));
  const month = Number(period.start.slice(5, 7));
  const nextMonth = month === 10 ? 1 : month + 3;
  const nextYear = month === 10 ? year + 1 : year;
  return quarterFromStart(iso(nextYear, nextMonth, 6))!;
}

export function availableQuarters(selectedStart?: string, count = 8): UkQuarter[] {
  const periods: UkQuarter[] = [];
  let period = currentUkQuarter();
  for (let index = 0; index < count; index += 1) {
    periods.push(period);
    period = nextUkQuarter(period);
  }
  const selected = selectedStart ? quarterFromStart(selectedStart) : null;
  if (selected && !periods.some(item => item.start === selected.start)) periods.unshift(selected);
  return periods;
}

export function isRealCalendarDate(value: string): boolean {
  if (!/^\d{4}-\d{2}-\d{2}$/.test(value)) return false;
  const [year, month, day] = value.split('-').map(Number);
  const date = new Date(Date.UTC(year, month - 1, day));
  return date.getUTCFullYear() === year && date.getUTCMonth() === month - 1 && date.getUTCDate() === day;
}

export function isDocumentPeriodValid(document: Pick<QuarterDocument, 'quarterStart' | 'quarterEnd'>): boolean {
  const period = quarterFromStart(document.quarterStart);
  return Boolean(period && period.end === document.quarterEnd);
}
