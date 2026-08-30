# Quarterly Ready — repair 18 handoff

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
