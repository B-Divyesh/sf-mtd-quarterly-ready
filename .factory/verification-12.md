# Independent verification 12 — FAIL

**Tested candidate:** `0323fdc3fcd77360467633488362bb4b32cef2de`
**Live URL:** <https://mtd-quarterly-ready.sociobot.in>
**Verification date:** 2026-08-29

## Decision

**FAIL — not releasable for the researched job.** The live frontend and backend identify as the requested candidate, and the local build and claim suite pass. The deployed backend can lose access to real records and accountant links when traffic reaches another replica. It also exceeds its documented per-client rate allowance and has no approved HMRC integration, so it cannot complete the required record-to-quarterly-submission job.

## First-read gate — PASS

A cold, unauthenticated live visit says **“Turn records into a checked quarterly update”**, names **UK sole traders, tutors and landlords**, and has the one-click **“Try it with sample data”** action beside **“Opens a private sample quarter. No account needed.”** The first screen therefore answers what it does, for whom, and what to click first in plain words. It is visible at desktop and 390×844 mobile.

Evidence: `verification-artifacts/verification-12-live-cold-desktop.png` and `verification-artifacts/verification-12-live-cold-mobile-390.png`.

## Required claims gate — PASS locally

`.factory/claims.json` is present and contains 21 registered claims. From the clean checkout, after `npm ci`, every listed command was run against the shipped demo entry point. The five Rust claim commands passed individually. The aggregate local browser suite passed **42/42**, including every browser claim; the full local result also covered 11 Vitest and 13 Rust tests.

All registered IDs passed: `demo-isolation`, `demo-access`, `privacy-no-tracking`, `accountant-csv`, `quarter-review`, `free-quarter-persistence`, `csv-import`, `receipt-capture`, `receipt-locality`, `quarter-record-separation`, `hmrc-submission`, `conditional-submission`, `hmrc-handoff`, `accountant-link`, `accountant-link-expiry`, `server-licence-gate`, `encrypted-storage`, `audit-log`, `anonymous-page-count`, `offline-browser-copy`, and `paid-tier`.

The focused live demo/claim run passed 14/15 scenarios: isolation, no account, same-origin/no-cookie privacy behavior, CSV, category review, CSV import and invalid-row recovery, receipt storage/locality and quota recovery, HMRC handoff, demo read-only pack, offline reload, and checkout/free-export behavior. The one failure is the real-record persistence failure below.

## Local quality gates — PASS

```text
npm ci                                                    PASS (60 packages; 0 vulnerabilities)
npm test                                                  PASS (11 Vitest, 13 Rust, 42 Playwright)
cargo fmt -- --check                                      PASS
cargo clippy --all-targets -- -D warnings                 PASS
BUILD_SHA=0323fdc... cargo build --release                PASS
PORT=4199 ./target/release/quarterly-ready                PASS
```

The release executable started with only `PORT` supplied, generated its encryption key, returned the candidate build SHA from `/health`, and shut down cleanly. The production frontend build is 44.69 KB JavaScript (14.62 KB gzip) and 21.67 KB CSS (5.33 KB gzip), within the applicable budgets.

## Live identity, UX, privacy, accessibility, and PWA evidence

- `/health` returned 200 with build SHA `0323fdc3fcd77360467633488362bb4b32cef2de`, `safe_qa_fixtures:true`, and `hmrc_integration_configured:false`.
- SHA-256 values for live `index.html`, hashed JS, and hashed CSS exactly matched the locally built candidate files.
- `EXPECTED_BUILD_SHA=0323fdc... npm run verify:live` passed its checkout, malformed-request, synthetic round-trip, fixture, header, and single-process limit checks. It observed 40 reads / 12 writes in that run; the later three-replica reproduction below is the authoritative external result.
- The cold page and live demo made only same-origin product requests and set no cookies. The live demo request-log claim passed. No console or page errors occurred in cold desktop/mobile, reduced-motion, PWA, or boundary-input checks.
- Live Axe scans on `/`, `/demo`, `/privacy`, and `/terms` found zero serious or critical violations. Keyboard demo entry, internal-link crawl, route metadata, mobile 390 px layout, 44 px targets, and cold records loading passed. At 390 px, `scrollWidth === clientWidth === 390`; the focused primary link has a 3 px teal outline. Reduced motion reports 0.00001 s transitions and animations.
- The service worker is active at `/sw.js`; `registration.update()` retained the active controller. With network disabled, `/demo` reloaded with “Offline — browser copy active” and the sample records.
- HTML and service worker responses are `no-cache`; the hashed JS is `public, max-age=31536000, immutable`. Responses include CSP with `frame-ancestors 'none'`, HSTS, `nosniff`, strict-origin referrer policy, and a restrictive permissions policy.
- End-to-end input exercise: £0 produced “The amount must be between £0.01 and £1,000,000.”; correcting it to £1,000,000 saved the transaction. The live suite also proved invalid CSV is atomic and a corrected CSV imports.

There is no sign-in route, so the Entra tenant requirement is not applicable. This is a web backend, not a library or CLI.

## Release-blocking defects

### P1 — real records and live accountant links are split across replicas

The actual Container App template in resource group `sociobot` has `minReplicas: 1`, `maxReplicas: 3`, no `/data` mount, and no volumes. `npm run verify:topology` fails accordingly. After load, Azure reported three simultaneously running replicas.

A fresh workspace PUT returned 200. Of 30 immediate GETs using the same workspace ID, only **8** returned the saved document and **22** returned `{"document":null}` (all HTTP 200). The live registered `@claim:free-quarter-persistence` / `@claim:quarter-record-separation` test fails reproducibly for the same reason: a server document just saved for the current or next quarter is absent on the subsequent API read.

This breaks the brief's transaction capture, durable record keeping, accountant-link storage, encryption-key continuity, and audit trail. A container-local SQLite database must not be exposed behind multiple replicas.

**Required:** mount durable Azure Files (or use a shared database) at `/data`, keep both replica bounds at one until state and the limiter are distributed, then prove records and links survive routing, restart, and revision replacement.

### P1 — documented rate allowance is not enforced externally

With the three replicas active, a single fresh client burst received **120** non-429 read responses before 30 `429` responses and **36** write responses before 24 `429` responses. Every 429 correctly included `Retry-After: 1`, but the documented allowance is 40 reads / 12 writes per client per second. The three process-local limiters triple that allowance.

**Required:** use one replica while local rate limiting is used, or use a shared limiter keyed on the first `X-Forwarded-For` hop; re-test externally at 40 reads / 12 writes and 429 plus `Retry-After` thereafter.

### P1 — approved HMRC submission is unavailable

The researched minimum product requires an HMRC-compatible submission through an approved integration after human review. Live `/health` reports `hmrc_integration_configured:false`; `npm run verify:release` fails immediately with `Error: production has no approved HMRC integration configured`. The UI honestly offers a reviewed handoff instead, but that is an incomplete fallback, not fulfilment of the brief.

**Required:** configure the approved integration URL and token through managed secret references and prove a provider-approved sandbox submission behind the human-review gate. Until then, do not mark this candidate as satisfying the record-to-submission product contract.

## Scope and repository state

No product source or deployment configuration was changed by this verification. Pre-existing `graphify-out/` worktree changes were left untouched. This report, the updated handoff, and the two new verification screenshots are the only intended verifier artifacts.
