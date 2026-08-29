import { describe, expect, it } from 'vitest';
import { receiptStorageMessage } from './receipts';

describe('receipt storage errors', () => {
  it('turns quota exhaustion into a recoverable user action', () => {
    const message = receiptStorageMessage(new DOMException('quota', 'QuotaExceededError'));
    expect(message).toContain('does not have enough space');
    expect(message).toContain('transaction was not changed');
    expect(message).toContain('try again');
  });

  it('does not expose raw IndexedDB errors', () => {
    expect(receiptStorageMessage(new DOMException('internal browser detail', 'AbortError')))
      .toBe('The receipt could not be saved in this browser. The transaction was not changed. Try again.');
  });
});
