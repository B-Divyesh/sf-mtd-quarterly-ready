import process from 'node:process';
import { randomUUID } from 'node:crypto';

const origin = new URL(process.env.VERIFY_ORIGIN || 'http://127.0.0.1:4173');
const rounds = Number.parseInt(process.env.CONCURRENCY_ROUNDS || '2', 10);

if (!Number.isInteger(rounds) || rounds < 1 || rounds > 10) {
  throw new Error('CONCURRENCY_ROUNDS must be an integer from 1 to 10.');
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function probeFor(round, index) {
  const workspace = randomUUID();
  const marker = `Concurrent durability probe ${round + 1}/${index + 1} ${workspace}`;
  return {
    workspace,
    marker,
    document: {
      schemaVersion: 1,
      businessName: 'Concurrent durability probe',
      quarterLabel: '6 April to 5 July 2026',
      quarterStart: '2026-04-06',
      quarterEnd: '2026-07-05',
      figuresReviewed: false,
      packDownloaded: false,
      markedReady: false,
      updatedAt: new Date().toISOString(),
      transactions: [{
        id: `concurrent-${round}-${index}-${workspace}`,
        date: '2026-04-09',
        description: marker,
        amountPence: index + 1,
        kind: 'income',
        category: 'Sales',
      }],
    },
  };
}

async function runRound(round) {
  const probes = Array.from({ length: 10 }, (_, index) => probeFor(round, index));
  const writes = await Promise.all(probes.map((probe, index) => fetch(new URL('/api/workspace', origin), {
    method: 'PUT',
    headers: {
      'content-type': 'application/json',
      'x-workspace-id': probe.workspace,
      'x-forwarded-for': `198.51.${round + 1}.${index + 1}`,
    },
    body: JSON.stringify({ document: probe.document }),
  })));
  assert(writes.every(response => response.status === 200), `round ${round + 1} write statuses were ${writes.map(response => response.status).join(',')}`);

  // Match independent verification 19: do not trust the acknowledgements
  // until every write has completed and the delayed reads all see their own
  // original document.
  await new Promise(resolve => setTimeout(resolve, 1500));
  const reads = await Promise.all(probes.map((probe, index) => fetch(new URL('/api/workspace', origin), {
    headers: {
      'x-workspace-id': probe.workspace,
      'x-forwarded-for': `2001:db8:${round + 1}::${index + 1}`,
    },
  }).then(async response => ({ response, body: await response.json() }))));
  const missing = reads.flatMap(({ response, body }, index) => (
    response.status === 200 && body.document?.transactions?.[0]?.description === probes[index].marker ? [] : [index]
  ));
  assert(missing.length === 0, `round ${round + 1} lost acknowledged workspaces at indexes ${missing.join(',')}`);
  return probes.length;
}

let preserved = 0;
for (let round = 0; round < rounds; round += 1) preserved += await runRound(round);

console.log(JSON.stringify({
  origin: origin.origin,
  rounds,
  concurrent_writes_per_round: 10,
  acknowledged_documents_preserved: preserved,
  delayed_read_ms: 1500,
  status: 'ok',
}));
