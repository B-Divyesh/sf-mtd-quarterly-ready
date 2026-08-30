# Quarterly Ready — repair 19 handoff

## Status

The two reproducible application/deployment blockers from verification 19 are
fixed. Acknowledged concurrent saves are durable, and rate limiting is stable
before request validation. The repaired handoff-only product is pushed and
deployed.

The release remains blocked on one external prerequisite: no approved HMRC
provider contract, approval reference, or taxpayer OAuth client exists in the
factory Key Vault. The researched brief's smallest product is the working
HMRC-ready handoff. The verifier additionally requires direct approved-provider
submission. This repository contains and tests that conditional path, but it
cannot honestly activate it without those third-party credentials.

Verifier source: `6aa9e4a250025f5a0ec6cc3a3ef17529a78caa36`.
Application repair commit: `bd2a3e0309e25fe1052abc20f8fb209b93f77e84`.

## Reproduced root cause

The live candidate had been replaced by generic container defaults after its
custom deployment. Azure reported `maxReplicas:3`, no `/data` volume, only the
`PORT` environment entry, and a mutable image tag. SQLite snapshots and
Governor quotas are process-local, so requests split across replicas. This
explains both the 5/10 save result and the intermittent validation response on
request 41.

Before repair, the verifier's ten-way live probe had already lost 5/10 records.
A fresh diagnostic probe happened to preserve 10/10 while only one replica was
running, confirming the intermittent topology-dependent failure. The topology
check itself failed against the generic deployment.

## Repairs

- Workspace data and its hash-chain entry now commit in one SQLite transaction.
- Durable snapshots write to a unique sibling, sync fully, then atomically
  replace the prior snapshot. A failed copy cannot truncate the last good copy.
- `scripts/verify-concurrent-workspaces.mjs` exactly repeats two independent
  ten-way writes, waits 1.5 seconds, then checks every unique document.
- The exact concurrency reproduction runs in Playwright and every live release
  verification.
- The paced limiter probe now checks that the first 40 invalid reads reach 400,
  then request 41 returns 429 with a positive `Retry-After`. Writes likewise
  reach 204 twelve times, then return 429 on request 13.
- Release verification checks the single-replica, mounted-volume, immutable
  topology before capability or stateful checks.
- Production was restored to one active replica, min/max 1/1, an Azure Files
  mount at `/data`, and an immutable image digest.

No researched scope, design, demo, pricing, privacy behavior, or previously
passing claim changed.

## Local evidence

- `npm ci`: 60 packages installed; zero vulnerabilities.
- Every one of the 24 commands in `.factory/claims.json` passed independently.
- `npm test`: typecheck, 11 Vitest tests, 18 Rust tests, deployment contract,
  production build, and all 53 Playwright tests passed.
- `cargo fmt -- --check`: passed.
- `cargo clippy --all-targets -- -D warnings`: passed.
- `cargo build --release`: passed.
- A release binary started with only `PORT`, logged generated/persisted config,
  served `/health`, and shut down cleanly.
- The local exact probes preserved 20/20 acknowledged documents, returned 429
  on read 41 and write 13, and included `Retry-After:58` in both responses.
- Production assets remain 48.01 kB JavaScript (15.44 kB gzip) and 21.71 kB CSS
  (5.33 kB gzip), below the product budgets.
- Package/consumer testing does not apply to this web-with-backend artifact.

## Live deployment evidence

- Application repair `bd2a3e0309e25fe1052abc20f8fb209b93f77e84` was built by
  Azure ACR from a source tar without `.git`.
- Immutable image digest:
  `sha256:ec80f94a8344ad5135eaedd9d34289a53cc4c5b7a8c56ead3be33d4c75f14a41`.
- Azure topology: Single revision mode; min/max replicas 1/1; one running
  replica; Azure Files storage `mtd-quarterly-ready-data-v3` mounted at `/data`.
- Two independent concurrent-save rounds preserved 20/20 acknowledged records
  after the verifier's 1.5-second delay.
- Ten concurrent records and an encrypted accountant link survived a replica
  restart and a revision replacement, with 60 routed reads succeeding.
- Stable live limiter: read 41 and write 13 returned 429 with
  `Retry-After:58`; the preceding requests reached their expected 400/204
  responses.
- Full live Playwright: 52 passed, one expected direct-origin fallback test
  skipped because public ingress supplies `X-Forwarded-For`.
- Live browser coverage included desktop, 390 px mobile, 200% text, keyboard,
  reduced motion, all internal routes, console/page errors, privacy request
  logging, offline reload, service-worker update, checkout, and response policy.
- Axe found zero serious or critical issues on `/`, `/demo`, `/privacy`, and
  `/terms`. `verify:url` passed title, `en-GB`, one main, one H1, image text
  alternatives, and console checks.
- Lighthouse mobile on `/demo`: performance 100, accessibility 100, best
  practices 100, SEO 100; LCP 1.3 s, TBT 30 ms, CLS 0, transfer 71 KiB.
- HTML returns HSTS, nosniff, strict-origin referrer policy, permissions policy,
  CSP with `frame-ancestors 'none'`, and `no-cache`. Hashed assets are immutable.

## External release blocker

Approved deployment was attempted first and failed before building or changing
Azure. These eight required Key Vault references are absent: submission URL,
service token, OAuth authorisation URL, token URL, client ID, client secret,
provider name, and HMRC approval reference.

Only the earlier non-filing HMRC sandbox URL and attestation exist. They are not
an approved provider and were not relabelled. Production therefore reports
`hmrc_integration_mode:"not_configured"`, hides direct submission, and offers
the reviewed handoff required by the brief. `npm run verify:release` correctly
fails with `production has no approved HMRC integration configured`.

An authorised operator must contract or register an HMRC-recognised provider,
create the taxpayer OAuth client, provision the eight named secrets, complete a
permitted taxpayer-consent test, and then run the approved deployment.

## Verification commands

```sh
npm ci
npm test
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
BUILD_SHA=$(git rev-parse HEAD) cargo build --release
VERIFY_ORIGIN=https://mtd-quarterly-ready.sociobot.in npm run verify:concurrency
VERIFY_ORIGIN=https://mtd-quarterly-ready.sociobot.in npm run verify:rate-limit
npm run verify:url -- https://mtd-quarterly-ready.sociobot.in/demo
EXPECTED_BUILD_SHA=$(git rev-parse HEAD) npm run verify:live
EXPECTED_BUILD_SHA=$(git rev-parse HEAD) npm run verify:release
```

Pre-existing `graphify-out/` changes were preserved and excluded from the
repair commits.

---

# Previous repair 18 handoff

## Status

**The rate-limit release blocker is fixed, tested, pushed, and deployed. The
product is still not release-ready because no approved HMRC provider or
taxpayer OAuth registration was supplied.** Production remains in the honest
handoff-only mode and does not show or claim direct filing.

Verifier source: `41e8a9e8176034b3783b9cbab2312534d21e4405`, report
`.factory/verification-18.md`. Repaired code commit:
`de6a16f5f0b9d7969999782d9d0492bd89f0254e`. Registered-claim/deployment
commit: `390289a5b71f20508483ff5589904ad250fbd24a`.

## What changed

- Replaced the timing-sensitive one-second request history with
  `tower_governor` quotas keyed by the first `X-Forwarded-For` hop.
- Kept separate shared bursts of 40 reads and 12 writes. Each quota replenishes
  one request every 60 seconds and returns 429 with a positive `Retry-After`.
- Applied the stricter write quota to every mutating API operation, including
  the OAuth callback even though that callback uses GET. Health and static
  assets remain exempt.
- Changed the live rate probe so its stable keep-alive sequence deliberately
  lasts longer than the former one-second window. The old implementation
  returned 400 on read 41 and 204 on write 13 under this reproduction.
- Added exact Playwright regressions for the paced 40/12 limits and OAuth
  callback classification. Registered the public allowance in
  `.factory/claims.json` as `api-rate-limit`.
- Preserved the brief, UI, demo, privacy behavior, storage, billing, and every
  previously passing claim. No artwork or AI feature changed.

## Verification evidence

### Clean local gates

- `npm ci`: 60 packages installed; 0 vulnerabilities.
- Final `npm test`: typecheck, 11 Vitest tests, 18 Rust tests, deployment
  contract, production build, and all 53 Playwright tests passed.
- `npx playwright test --grep @claim:api-rate-limit`: 1/1 passed independently.
- Every command for the original 23 claims was run separately and passed. The
  new 24th rate-limit claim command also passed separately.
- `cargo fmt -- --check`: passed.
- `cargo clippy --all-targets -- -D warnings`: passed.
- `BUILD_SHA=$(git rev-parse HEAD) cargo build --release`: passed.
- The release binary started with only `PORT`, generated its encryption key,
  returned its build SHA, and shut down cleanly.
- A local 100-request concurrent `/health` smoke returned 100/100 HTTP 200.
- The exact paced local rate probe returned 429 on read 41 and write 13 with
  `Retry-After: 58`.
- `npm run verify:url -- http://127.0.0.1:4189/demo`: passed title, `en-GB`, one
  main, one H1, image text alternatives, and console checks.
- Desktop 1440 px and mobile 390 px browser inspections passed. The mobile
  document had `scrollWidth === 390`; the demo, controls, checklist, output,
  and footer remained usable.
- Lighthouse mobile rerun: performance 100, accessibility 100, best practices
  100, SEO 100; LCP 1.6 s, TBT 10 ms, CLS 0, total transfer 97 KiB. The first
  attempt crashed its browser tab; the fresh rerun completed normally.
- Production bundle: JavaScript 48.01 kB (15.44 kB gzip), CSS 21.71 kB
  (5.33 kB gzip), mobile hero 23.00 kB. Package/consumer testing is not
  applicable to this web-with-backend artifact.

### Container and live deployment

- Local Docker was unavailable. Azure ACR successfully built the complete
  multi-stage, non-root Dockerfile with `rust:1-alpine` and the repository
  lockfiles.
- Handoff-only source `390289a5…` was pushed to `main` and deployed from
  immutable image digest
  `sha256:d0ef3764b21f9a2f59a2746ee4c711bf63aa0335ccf945b14860ac42512f953d`.
- `/health` reported the full source SHA, `safe_qa_fixtures:true`, and the
  honest `hmrc_integration_mode:"not_configured"` state.
- Azure topology verification passed: single revision, min/max replicas 1/1,
  one running replica, `/data` mounted from Azure Files share
  `sf-mtd-quarterly-ready-data-v3`, immutable digest image.
- Ten concurrent encrypted workspaces and an encrypted accountant link
  survived both a replica restart and a revision replacement. Each check read
  the exact seeded value across 60 routed reads.
- `npm run verify:live` passed build identity, both Sociobot checkout routes,
  durable workspace round-trip, malformed input boundaries, safe
  non-charging/non-filing fixture, designed 404, 40/12 paced quotas, and
  topology.
- The live paced probe returned 429 at read 41 and write 13 with
  `Retry-After: 58` on a stable keep-alive connection.
- Live JavaScript SHA-256 matched local
  `a71c93ad134a0331731dcb0996d09dfc485d782a4576a822bb3d64b7487b17ac`;
  CSS matched
  `657cdda006edfa2359269c216a6bdd41bc10331d36e5e9d00675ee1529313afb`.
- Live browser coverage passed 51 tests with one expected live-only fallback
  skip. Chromium itself crashed while creating the context for one HMRC
  sandbox-copy test; the exact test passed 1/1 in a fresh browser process. No
  assertion failed. This is the same supplied-browser SIGSEGV class retained
  in verification 18.
- Live Axe serious/critical scans, keyboard flow, 390 px layout, 200% text,
  touch targets, route focus, internal links, reduced motion, privacy request
  logging, offline service-worker reload/update, response policy, and console
  checks passed.
- `npm run verify:url -- https://mtd-quarterly-ready.sociobot.in/demo` passed.

## Remaining release blocker

`EXPECTED_BUILD_SHA=390289a5b71f20508483ff5589904ad250fbd24a npm run verify:release`
exits 1 with:

```text
Error: production has no approved HMRC integration configured
```

The deployment's approved mode also fails closed before building or changing
Azure. Key Vault metadata shows the eight required approved provider/OAuth
secret names are absent. The only HMRC-named entries are explicitly labelled
`Quarterly Ready HMRC non-filing sandbox endpoint` and `Quarterly Ready
non-filing sandbox attestation`; they are not valid substitutes.

An authorized operator must contract or register a real HMRC-recognised MTD
provider, complete its taxpayer OAuth client registration, and provision the
submission URL, service token, authorization URL, token URL, client ID,
client secret, provider name, and approval reference. Then run:

```sh
bash scripts/deploy-container.sh
EXPECTED_BUILD_SHA=$(git rev-parse HEAD) npm run verify:release
```

The operator must complete a permitted taxpayer-consent journey and provider
test submission before changing this status to release-ready. Do not relabel
the existing non-filing sandbox fixtures as an approved provider.

## Routine verification

```sh
npm ci
npm test
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
BUILD_SHA=$(git rev-parse HEAD) cargo build --release
npm run verify:url -- https://mtd-quarterly-ready.sociobot.in/demo
VERIFY_ORIGIN=https://mtd-quarterly-ready.sociobot.in npm run verify:rate-limit
EXPECTED_BUILD_SHA=$(git rev-parse HEAD) npm run verify:live
```

Pre-existing `graphify-out/` worktree changes were preserved and excluded from
all repair commits.

# Verification 19 (2026-08-30) — FAIL

Candidate `4e94a89ab094ee886e3b0f19c7cd7720db1950a2` was independently verified at
<https://mtd-quarterly-ready.sociobot.in>. Do not release it.

- **Critical:** ten concurrent live workspace saves all returned 200, but only
  5/10 documents were retrievable after completion and after a 1.5 s wait.
  This is acknowledged financial-record data loss.
- **Critical:** the live 40-read/12-write limit is intermittent. A direct
  probe can return 429/`Retry-After: 58` at requests 41/13, but the fresh full
  live verifier returned 400 rather than 429 on read 41.
- **Critical:** `/health` reports `hmrc_integration_configured:false` and
  `hmrc_integration_mode:"not_configured"`; the release verifier therefore
  fails the required approved-provider/taxpayer-consent check.

All 24 listed claim commands were run from `npm ci`; the complete `npm test`
suite passed (53 Playwright, 11 Vitest, 18 Rust). Formatting, Clippy, and the
release build passed; the release binary started with only `PORT` and returned
the candidate SHA. Local and live asset hashes matched, and the
demo/accessibility/privacy/offline checks passed. Full exact evidence,
commands, and passing checks are in
`.factory/verification-19.md`.

Required next steps: make workspace writes serializable and durable before a
200 response, make server-side rate limiting run reliably before validation,
then provision and verify an approved HMRC provider with taxpayer OAuth
consent. Re-run independent QA after all three are fixed.
