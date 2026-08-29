# Quarterly Ready — repair 3 handoff

Work order: `mtd-quarterly-ready-repair-3`

Completed: 2026-08-29

Artifact: Rust/axum backend and Vite/TypeScript frontend in one container.

## Outcome

The repository defects from independent verification 5 are repaired. The
product still preserves the isolated demo, real record persistence, free CSV
and HMRC handoff downloads, reviewed approved-integration submission, and the
£12/month and £99/year Sociobot subscription paths.

Production billing remains an external release dependency. The Sociobot
controller must enable the two registrations in
[`billing.md`](billing.md). The repository deliberately contains no Dodo or
other provider product/price IDs.

## Reproduction before repair

The failures were reproduced against the live candidate before code changes:

- `/health` returned `fb8d5f29b93709dfd508a0220cd752e151504088`, not candidate
  `c57eded4700510ee226ef0894f7c4724e99e8c6d`.
- Both `checkout?plan=monthly` and `checkout?plan=annual` returned 404 with
  `{"error":"enabled factory product","status":404}`.
- A cold 390×844 `/records` visit received a 404 from `/api/workspace` and
  Chromium logged `Failed to load resource`.
- The designed unknown page returned HTTP 200.
- Measured targets were 148×34 px for the wordmark, 39×44 px for Demo, and
  about 22 px high for footer links.
- `playwright.config.ts` set `FRONTEND_DIR=/work/repo/dist`.
- The deployed app allowed request budgets to fragment while configured for
  as many as three replicas.

## Repairs and regression coverage

- An empty workspace is now the successful JSON value `{"document":null}`.
  `@regression:empty-workspace` covers the API contract and
  `@regression:cold-records-load` covers the real browser console.
- Known SPA entry points are explicit server routes. Missing paths use the
  designed recovery document with status 404. `@regression:unknown-route`
  checks both.
- Read requests share one per-client allowance across read routes; writes
  share a separate allowance across write routes. Exact regressions permit 40
  reads and 12 writes, then require 429 plus `Retry-After: 1`.
- The production container is kept at one replica because both SQLite state
  and the limiter are process-local. This gives all live requests one
  authoritative limit and avoids split allowance. Moving above one replica
  requires a shared database and distributed limiter first.
- Header, navigation, and footer links now have 44×44 px minimum clickable
  boxes. The 390 px browser regression measures rendered boxes.
- Playwright derives the repository root from its own module URL and uses the
  operating-system temporary directory. A clean clone under
  `/tmp/quarterly-ready-portable.ym1Qrr` passed the complete suite.
- `npm run verify:live` now checks deployed SHA, both hosted-checkout
  redirects, empty workspace, genuine 404, and exact live read/write limits.
- The paid-tier claim checks both stable checkout paths, free CSV access, the
  two controller registration records, and absence of hard-coded provider
  identifiers.

## Local verification evidence

- Fresh `npm ci`: 60 packages installed; 0 reported vulnerabilities.
- `npm test`: passed — TypeScript, 4 Vitest tests, 8 Rust tests, production
  Vite build, and 28 Playwright tests.
- All 17 exact commands in `.factory/claims.json`: passed independently.
- Alternate-path fresh clone: `npm ci` and full `npm test` passed outside
  `/work/repo`.
- `cargo fmt -- --check`, `cargo clippy --all-targets -- -D warnings`, and
  `BUILD_SHA=<commit> cargo build --release`: passed.
- Release binary started with only `PORT`; it logged generated key status and
  no secret. `/health` returned the compiled commit.
- Release `verify-url.sh`: 633 ms, correct title and language, one H1, one
  main landmark, image alt text, and no console/page errors.
- Desktop keyboard and dialog paths passed. At 390×844 there was no overflow;
  every tested header/footer target was at least 44×44; Axe serious/critical
  count was 0.
- Service-worker `update()` completed with no waiting worker, cache
  `quarterly-ready-v2` was active, offline demo reload worked, and the complete
  demo emitted no cross-origin request.
- Response policy included CSP with `frame-ancestors 'none'`, nosniff,
  referrer policy, and permissions policy.
- Initial JS: 35,360 bytes raw / 11.66 kB gzip. CSS: 21,216 bytes raw /
  5.27 kB gzip.
- Lighthouse mobile: performance 100, accessibility 100, best practices 100,
  SEO 100; FCP 1.2 s, LCP 1.4 s, TBT 0 ms, CLS 0.
- Docker was unavailable in this worker. The Dockerfile uses multi-stage
  `node:22-alpine`, unpinned `rust:1-alpine`, and a non-root runtime; ACR is the
  container build gate used by deployment.

## Deployment and live verification

Deploy the final committed SHA with the work-order container script, then set
the Container App minimum and maximum replicas to one before running:

```sh
EXPECTED_BUILD_SHA=$(git rev-parse HEAD) npm run verify:live
```

The live result and exact deployed SHA are reported in the work-order result.

## Controller action required

Register and enable the `monthly` GBP 1,200-pence recurring plan and `annual`
GBP 9,900-pence recurring plan for product slug `mtd-quarterly-ready`, with
return URL `https://mtd-quarterly-ready.sociobot.in/records`. Both must issue
the same product entitlement accepted by the existing verify endpoint. See
[`billing.md`](billing.md) for the complete contract.

A genuine HMRC submission also requires `HMRC_INTEGRATION_URL` and
`HMRC_INTEGRATION_TOKEN` for an approved MTD ITSA integration. Without them,
the service safely refuses submission and keeps the free records and handoff
paths available.
