# Quarterly Ready — verification 16 handoff

## Release status: FAIL

**Verified candidate / deployed build:** `9c0c66f8427e28503d4ed8789a8de7496f9efc3f`
**URL:** <https://mtd-quarterly-ready.sociobot.in>

Independent QA is recorded in [`.factory/verification-16.md`](verification-16.md).
The candidate is deployed and its local quality gates pass, but it must not be
released as the researched end-to-end MTD product. Live `/health` reports no
approved HMRC integration (`hmrc_integration_configured: false`), so the app
truthfully offers only a reviewed handoff rather than a reviewed submission.
That fails the product brief's essential approved-integration submission job.

## What was verified

- Cold first-read passed: what it does, who it is for, and the one-click
  sample-data demo are all explicit.
- Every claim declared in `.factory/claims.json` passed locally from the demo
  entry path; `npm test` passed (49 Playwright tests, 11 Vitest, 16 Rust).
- Typecheck, production Vite build, formatting, Clippy and release Rust build
  passed. Initial JS is 14.99 kB gzip; CSS is 5.33 kB gzip.
- Live build identity exactly matches the candidate; live save/read,
  invalid-input, safe-fixture and checkout checks passed.
- Live privacy, headers, desktop/390 px behaviour, keyboard/focus,
  reduced-motion, service-worker offline reload and Axe serious/critical checks
  passed. One persistent client was limited at 40 reads / 12 writes per second
  with `429 Retry-After: 1`.

## Remaining work

1. Configure and verify a real approved HMRC integration with the required
   consent/authorisation path. Then run
   `EXPECTED_BUILD_SHA=<deployed-sha> npm run verify:release` successfully.
2. Stabilise the checked-in live Playwright rate-limit regression through the
   ingress; it does not retain a stable observed client identity over its
   HTTP/1.1 transport in this QA environment, though a persistent-client probe
   demonstrates the endpoint policy.
3. Container build was not run because Docker is unavailable in this QA
   container.

## Verify

```sh
npm ci
npm test
EXPECTED_BUILD_SHA=9c0c66f8427e28503d4ed8789a8de7496f9efc3f npm run verify:live
EXPECTED_BUILD_SHA=9c0c66f8427e28503d4ed8789a8de7496f9efc3f npm run verify:release
```

---

# Previous repair 15 handoff

## Deployment and release status

**Deployed repair:** <https://mtd-quarterly-ready.sociobot.in>
**Source / live build SHA:** `958d708d0022894d2c231f1a09eea1799f2f30ed`
**ACR build:** `ch160` — succeeded 2026-08-29 18:44 UTC
**Container Apps revision:** `sf-mtd-quarterly-ready--0000048` — Healthy

The deployed configuration now has one active revision, exactly one running
replica (`minReplicas: 1`, `maxReplicas: 1`), and a read/write Azure Files
mount at `/data`. The server keeps SQLite on its local filesystem and persists
an encrypted snapshot plus its encryption key to that durable mount after each
serialized mutation. The deployer stops the predecessor before starting a
successor, so two SQLite writers cannot overlap.

This is a truthful **handoff-only** deployment. It fixes the verifier's live
data-loss, split-rate-limit, deployment-identity, and missing URL-check
findings. It is not a filing release: an approved HMRC provider integration is
not configured, so direct submission is hidden and the app offers the reviewed
HMRC handoff instead.

## Repairs completed

1. **Durable real records and one authoritative limiter.** Applied the
   committed Azure Files `/data` mount and `Single`, 1/1-replica topology to
   the live Container App. The live verifier now proves a saved workspace is
   returned immediately, and the public ingress enforces exactly 40 API reads
   or 12 writes per client per second before `429` with `Retry-After: 1`.
2. **Deployable identity.** Pushed and deployed the exact repair commit. The
   ACR build receives the full commit through `BUILD_SHA`; `/health` returns
   that same full SHA. The non-resolvable requested SHA from verification 15
   is superseded by this pushed, verifiable commit.
3. **Receipt-storage regression retained.** The existing IndexedDB receipt
   implementation and `@claim:receipt-capture` regression pass with three
   1.4 MB PDFs. Receipt bytes remain out of localStorage and server documents;
   quota errors preserve the form and announce recovery.
4. **Repeatable URL accessibility smoke check.** Added executable
   `scripts/verify-url.sh` plus `scripts/verify-url.mjs`, exposed as
   `npm run verify:url -- <url>`. It uses Chromium to assert a title, `lang`,
   exactly one `<main>`, one `<h1>`, alt attributes for all images, and no
   console/page errors. `@regression:verify-url-helper` executes it in the
   Playwright suite.
5. **Response-policy regression.** Added
   `@regression:response-policy`, covering CSP frame protection and connect
   policy, `nosniff`, referrer and permissions policies, and no-cache HTML/
   service-worker responses.

## Verification evidence

All local commands were run after a clean `npm ci` (60 packages, 0 reported
vulnerabilities) on 2026-08-29.

| Check | Result |
| --- | --- |
| `npm test` | PASS — typecheck, 11 Vitest tests, 16 Rust tests, deploy-contract check, production build, and 49 Playwright tests. |
| `cargo fmt -- --check` | PASS |
| `cargo clippy --all-targets -- -D warnings` | PASS |
| `BUILD_SHA=local-repair cargo build --release` | PASS |
| `npx playwright test --grep '@claim:'` | PASS — 13 browser claim tests covering all browser claim IDs. |
| Each declared Rust claim command | PASS — approved-integration payload/human review, 30-day links, encrypted storage, hash chain, anonymous page count, and non-filing sandbox. |
| Production Vite build | PASS — JS 46.24 kB / 14.99 kB gzip; CSS 21.71 kB / 5.33 kB gzip. |
| ACR production container build | PASS — `ch160`, including the committed multi-stage Dockerfile. |
| `npm run verify:topology` | PASS — `Single`, 1/1 replicas, one running replica, Azure Files mounted at `/data`. |
| `EXPECTED_BUILD_SHA=958d708… VERIFY_AZURE_TOPOLOGY=1 npm run verify:live` | PASS — exact identity, workspace save/read, malformed-input rejection, checkout, safe non-charging/non-filing fixture, 40/12 rate limits, and topology. |
| `npm run verify:url -- https://mtd-quarterly-ready.sociobot.in` and `/demo` | PASS — title, language, one main/H1, alt text, no browser errors. |
| Live Playwright | PASS — 48 passed, 1 documented ingress-only skip. Covers desktop, 390px mobile, keyboard, Axe serious/critical, privacy/cookies/requests, offline reload and service-worker update, response policies, records, receipt quota/recovery, and rate limits. |
| Live Lighthouse mobile | PASS — 100 performance, 100 accessibility, 100 best practices, 100 SEO; LCP 1,353 ms, TBT 3 ms, CLS 0, 94,171 bytes transferred. Evidence: `.factory/repair-15-evidence/lighthouse.json`. |

The mobile evidence in `.factory/repair-15-evidence/live-demo-mobile-390.png`
and desktop evidence in `.factory/repair-15-evidence/live-cold-desktop.png`
are captured from the deployed build.

## Known external prerequisite — still release-blocking for filing

`EXPECTED_BUILD_SHA=958d708… npm run verify:release` correctly exits 1 with
`production has no approved HMRC integration configured`. The required Key
Vault references `mtd-quarterly-ready-approved-hmrc-url` and
`mtd-quarterly-ready-approved-hmrc-token` do not exist. Two legacy secrets
with different names were intentionally not used because they were previously
identified as a non-filing sandbox; configuring them as a live provider would
be false and unsafe.

To make the researched end-to-end HMRC submission capability releasable, the
product owner must provision an approved MTD ITSA provider URL/token under
those exact Key Vault names, confirm taxpayer authority/consent and a safe
authorised acceptance path, then deploy in default `approved` mode and run
`EXPECTED_BUILD_SHA=<deployed-sha> npm run verify:release`. Until then,
Quarterly Ready is correctly limited to records, CSV/accountant exports, and a
reviewed recognised-software handoff.

## Run, verify, and deploy

```sh
npm ci
npm test
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
BUILD_SHA=dev cargo build --release

# Start locally in another terminal, then run the repeatable browser smoke check.
PORT=8080 cargo run
npm run verify:url -- http://127.0.0.1:8080/demo

# Requires Azure access.
npm run verify:topology
EXPECTED_BUILD_SHA=<sha> VERIFY_AZURE_TOPOLOGY=1 npm run verify:live

# Approved provider secrets are required; this is the normal filing release.
bash scripts/deploy-container.sh

# Explicit safe fallback when the approved provider is unavailable.
DEPLOYMENT_MODE=handoff-only bash scripts/deploy-container.sh
```

Pre-existing dirty `graphify-out/` files remain untouched.
