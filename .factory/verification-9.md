# Independent verification 9 — FAIL

**Tested candidate:** `0c99c04bc67fbd49e2403b97290569bb80bba607`  
**Live URL:** https://mtd-quarterly-ready.sociobot.in  
**Verification date:** 2026-08-29

## Release decision

**FAIL — P1 release blocker.** The live service identifies itself as candidate
`0c99c04bc67fbd49e2403b97290569bb80bba607`, but its required safe entitlement
fixture is disabled. `EXPECTED_BUILD_SHA=0c99c04bc67fbd49e2403b97290569bb80bba607 npm run verify:live`
fails at `/api/qa/entitlement` with HTTP 404 and:

```json
{"error":"The safe QA fixture is not enabled."}
```

This blocks independent proof of the subscription-gated accountant-link and
approved-integration submission paths without charging, filing, or obtaining a
real customer entitlement. It is a deployment/runtime failure, not a source
identity mismatch: `/health` returned the candidate SHA. The previous handoff's
claim that this fixture was enabled is not true for this deployment.

## First-read test (cold live landing page)

Passed. The first screen says: “Turn records into a checked quarterly update,”
names “UK sole traders, tutors and landlords,” and offers the one-click **Try
it with sample data** action with “Opens a private sample quarter. No account
needed.” The action opens the sample quarter directly.

## Claims contract

`.factory/claims.json` exists and declares 18 claims. From the clean candidate
checkout, after `npm ci`, every command in the manifest was run separately.
All 18 passed locally:

| Claims | Result |
| --- | --- |
| demo-isolation, demo-access, privacy-no-tracking, accountant-csv, quarter-review, free-quarter-persistence, csv-import, receipt-capture, hmrc-handoff, accountant-link, server-licence-gate, offline-browser-copy, paid-tier | Pass — each declared Playwright grep command |
| hmrc-submission, accountant-link-expiry, encrypted-storage, audit-log, anonymous-page-count | Pass — each declared `cargo test` command |

The passing local fixture is insufficient to accept the live release because
the candidate deployment returns 404 for that same required safe fixture.

## Local build and test evidence

All of the following passed from this checkout:

```sh
npm ci                         # 60 packages; 0 vulnerabilities
npm test                       # 9 Vitest, 13 Rust, 35 Chromium tests
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
BUILD_SHA=0c99c04bc67fbd49e2403b97290569bb80bba607 cargo build --release
```

Production frontend build output: JavaScript 41.02 kB (13.40 kB gzip) and CSS
21.67 kB (5.33 kB gzip). The live hashed JavaScript asset is the same
`index-BULAEdKy.js` produced locally. The release binary is 11,595,208 bytes.

The full browser suite exercised sample isolation, CSV export/import, receipt
attachment, category resolution, current/future-quarter persistence, invalid
CSV recovery without partial import, HMRC-handoff download, offline reload,
subscription gating, backend validation and persistence, keyboard dialog
operation, and the designed 404 path.

## Live deployment evidence

* `GET /health`: 200, `{ "status":"ok", "build_sha":"0c99c04bc67fbd49e2403b97290569bb80bba607" }`.
* The exact live verifier failed only at the required safe fixture as described
  above; consequently it could not prove the non-charging/non-filing live paid
  flow.
* Rate limits are live and shared by client IP: 40 reads accepted then 8/48
  requests returned 429; 12 writes accepted then 8/20 returned 429. Each 429
  had `Retry-After: 1`.
* Cold `/` and exercised `/demo` made same-origin requests only; browser
  cookies were empty; no console or page errors occurred. The normal page-view
  request is same-origin.
* Headers include HSTS, `X-Content-Type-Options: nosniff`, strict origin
  referrer policy, restrictive permissions policy, and CSP with
  `frame-ancestors 'none'`. Hashed JS is cached for one year immutable; the
  service worker is no-cache. The unknown-route response is a genuine 404.
* In a fresh 390px context `/demo` had `scrollWidth == clientWidth == 390`.
  Under reduced motion the tested control transition duration was `0.00001s`.
  Focus was visibly `rgb(11, 98, 92) solid 3px` with a 3px offset.
* Live Axe scans of `/`, `/demo`, `/privacy`, and `/terms` found no
  serious/critical violations. Each had one h1 and an appropriate route title.

## Defects

| Severity | Finding | Fresh evidence | Required resolution |
| --- | --- | --- | --- |
| P1 / release-blocking | Live safe entitlement fixture disabled | `npm run verify:live` against the exact candidate fails: `safe entitlement fixture returned 404`; direct `GET /api/qa/entitlement` returns the error above. | Deploy with the constrained safe QA fixture enabled, then rerun the exact live verifier against the resulting SHA. Do not treat a matching `/health` SHA as sufficient. |

## Scope notes

This is a web-with-backend product. It has no sign-in route, so no Entra flow
is applicable. It is a PWA; the local claim suite passed the service-worker
offline reload test. No product source was modified during verification.
