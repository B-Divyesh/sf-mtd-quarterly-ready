import http from 'node:http';
import https from 'node:https';
import process from 'node:process';
import { randomUUID } from 'node:crypto';

const origin = new URL(process.env.VERIFY_ORIGIN || 'http://127.0.0.1:4173');
const selectedKind = process.argv.includes('--kind')
  ? process.argv[process.argv.indexOf('--kind') + 1]
  : 'all';

if (!['all', 'read', 'write'].includes(selectedKind)) {
  throw new Error('--kind must be all, read, or write');
}

const transport = origin.protocol === 'https:' ? https : http;
const Agent = origin.protocol === 'https:' ? https.Agent : http.Agent;
const agent = new Agent({ keepAlive: true, maxSockets: 1, maxFreeSockets: 1 });
const pacedDelayMs = { read: 30, write: 100 };

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function send(path, method, clientIp) {
  return new Promise((resolve, reject) => {
    const request = transport.request(new URL(path, origin), {
      method,
      agent,
      headers: {
        connection: 'keep-alive',
        'x-forwarded-for': clientIp,
      },
    }, response => {
      response.resume();
      response.once('end', () => resolve({
        status: response.statusCode,
        retryAfter: response.headers['retry-after'],
        reusedSocket: request.reusedSocket,
      }));
    });
    request.once('error', reject);
    request.setTimeout(10_000, () => request.destroy(new Error(`Timed out waiting for ${path}`)));
    request.end();
  });
}

async function verify(kind, allowance) {
  // A single keep-alive agent and one socket make the public-ingress client
  // identity stable. Playwright's APIRequestContext may spread a burst over
  // several HTTP/1.1 connections, which does not exercise one client at the
  // ingress even when every request carries the same test header.
  const clientIp = `2001:db8:${randomUUID().replaceAll('-', '').slice(0, 4)}::42`;
  const responses = [];
  for (let index = 0; index <= allowance; index += 1) {
    if (index > 0) {
      await new Promise(resolve => setTimeout(resolve, pacedDelayMs[kind]));
    }
    responses.push(await send(
      kind === 'read' ? (index % 2 ? '/api/share/not-a-token' : '/api/workspace') : '/api/page-view',
      kind === 'read' ? 'GET' : 'POST',
      clientIp,
    ));
  }

  assert(
    responses.slice(0, allowance).every(response => response.status !== 429),
    `${kind} limit rejected a request before the ${allowance}-request allowance`,
  );
  assert(
    responses.slice(0, allowance).every(response => response.status === (kind === 'read' ? 400 : 204)),
    `${kind} requests did not reach the expected validation response before the allowance`,
  );
  assert(responses[allowance].status === 429, `${kind} request ${allowance + 1} returned ${responses[allowance].status}, expected 429`);
  const retryAfter = Number.parseInt(responses[allowance].retryAfter || '', 10);
  assert(Number.isInteger(retryAfter) && retryAfter > 0, `${kind} 429 response omitted a positive Retry-After value`);
  assert(responses.slice(1).some(response => response.reusedSocket), `${kind} probe did not reuse its single keep-alive connection`);

  return {
    allowance,
    first_limited_request: allowance + 1,
    retry_after: responses[allowance].retryAfter,
    paced_beyond_previous_one_second_window: true,
    stable_keep_alive_connection: true,
  };
}

async function verifyOauthCallbackWriteLimit() {
  const clientIp = `2001:db8:${randomUUID().replaceAll('-', '').slice(0, 4)}::43`;
  const allowance = 12;
  const admitted = [];
  for (let index = 0; index < allowance; index += 1) {
    if (index > 0) await new Promise(resolve => setTimeout(resolve, pacedDelayMs.write));
    admitted.push(await send('/api/page-view', 'POST', clientIp));
  }
  assert(
    admitted.every(response => response.status === 204),
    `OAuth callback quota setup did not admit all ${allowance} writes`,
  );

  await new Promise(resolve => setTimeout(resolve, pacedDelayMs.write));
  const callback = await send('/api/hmrc/consent/callback?state=missing', 'GET', clientIp);
  assert(callback.status === 429, `OAuth callback after ${allowance} writes returned ${callback.status}, expected 429`);
  const retryAfter = Number.parseInt(callback.retryAfter || '', 10);
  assert(Number.isInteger(retryAfter) && retryAfter > 0, 'OAuth callback 429 omitted a positive Retry-After value');

  return {
    shared_write_allowance: allowance,
    first_limited_request: allowance + 1,
    status: callback.status,
    retry_after: callback.retryAfter,
  };
}

try {
  const result = {};
  if (selectedKind === 'all' || selectedKind === 'read') result.read = await verify('read', 40);
  if (selectedKind === 'all' || selectedKind === 'write') {
    result.write = await verify('write', 12);
    result.oauth_callback = await verifyOauthCallbackWriteLimit();
  }
  console.log(JSON.stringify({ origin: origin.origin, ...result, status: 'ok' }));
} finally {
  agent.destroy();
}
