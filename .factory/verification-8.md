# Independent verification 8 — FAIL

**Candidate:** `2611ee3c3238aa16603e0212e950b3ddf7e1116d`  
**Live URL:** https://mtd-quarterly-ready.sociobot.in  
**Date:** 2026-08-29

## Decision

**FAIL — release-blocking deployment verification defect.** The deployed site is
the requested candidate and the user-facing demo is otherwise healthy, but the
repository's documented live verification command cannot complete against the
deployment. It fails at `/api/qa/entitlement` with HTTP 404. This prevents the
required safe verification of the subscription-gated accountant-link and HMRC
submission paths in the deployed environment.

## First-read test (cold live page)

Pass. In a new Chromium context at desktop size, the first screen says
“Turn records into a checked quarterly update,” says it is for “UK sole
traders, tutors and landlords” who need MTD records without a full accounting
suite, and shows one primary **Try it with sample data** action with the
adjacent explanation “Opens a private sample quarter. No account needed.” The
action opened `/demo` without an account.

## Required claims, run first from this checkout

After `npm ci` (60 packages, 0 vulnerabilities), every command listed in
`.factory/claims.json` was run separately and exactly as declared, before the
broader test suite.

| Result | Evidence |
| --- | --- |
| PASS (13 browser claim commands) | `npx playwright test --grep @claim:<id>` for `demo-isolation`, `demo-access`, `privacy-no-tracking`, `accountant-csv`, `quarter-review`, `free-quarter-persistence`, `csv-import`, `receipt-capture`, `hmrc-handoff`, `accountant-link`, `server-licence-gate`, `offline-browser-copy`, and `paid-tier`. |
| PASS (5 Rust claim commands) | `cargo test` for `claim_hmrc_submission_uses_an_approved_integration_after_human_review`, `claim_accountant_link_expiry`, `claim_encrypted_storage`, `claim_hash_chained_audit_log`, and `claim_anonymous_page_count`; each: 1 passed, 0 failed. |

## Local quality gates

| Check | Result |
| --- | --- |
| `npm test` | PASS: typecheck; 9 Vitest tests; 13 Rust tests; deployment-contract check; production Vite build; 35 Playwright tests, including accessibility, keyboard, invalid-input, backend, rate-limit, and claim coverage. |
| `npm run build` | PASS. Initial JS 41.02 kB / 13.40 kB gzip; CSS 21.67 kB / 5.33 kB gzip; mobile hero 23 kB. |
| `cargo build --release` | PASS; produced `target/release/quarterly-ready` (12 MB). |
| `cargo fmt -- --check` and `cargo clippy --all-targets -- -D warnings` | PASS. |
| Docker production build | Not run: `docker` is not installed in this QA container. |

## Deployed evidence

- `/health` returned `200 {"status":"ok","build_sha":"2611ee3c3238aa16603e0212e950b3ddf7e1116d"}`.
- The live root's hashed JS, CSS, hero images, and social image had the same
  SHA-256 hashes as a fresh local production build. The deployment therefore
  matches the candidate source, not merely its health label.
- Cold demo at both 1440×900 and 390×844: no console/page errors; one H1 and
  one main landmark; no horizontal overflow at 390 px. Keyboard test opened
  the demo; local full suite confirmed visible focus and keyboard-operable
  review dialog.
- Independent live Axe scan of `/demo`: **0 serious/critical violations**.
  The local full suite also found zero serious/critical violations on `/`,
  `/demo`, `/privacy`, and `/terms`.
- Representative demo flow passed live: categorising the unresolved transfer,
  rejected out-of-quarter CSV with unchanged row count, recovery message,
  service-worker readiness, offline reload showing “Offline — browser copy
  active,” and 390px layout.
- Privacy: fresh live demo had no cookies, no console errors, and only the
  product origin in its outgoing request log. Response headers include CSP
  with `frame-ancestors 'none'`, HSTS, nosniff, strict referrer policy, and
  restrictive permissions policy. Hashed JS/CSS have one-year immutable
  caching; shell and service-worker responses use `no-cache`.
- API allowance is enforced live when keyed by `X-Forwarded-For`: 40 reads
  returned HTTP 400 (missing workspace id) followed by 8 × HTTP 429 with
  `Retry-After: 1`; 12 writes returned HTTP 204 followed by 8 × HTTP 429 with
  `Retry-After: 1`.

## Defects

| Severity | Finding | Fresh evidence and required resolution |
| --- | --- | --- |
| P1 — release blocking | The live verification contract is broken. | `EXPECTED_BUILD_SHA=2611ee3c3238aa16603e0212e950b3ddf7e1116d npm run verify:live` reached the deployed candidate and failed with `Error: safe entitlement fixture returned 404`. The script requires `/api/qa/entitlement` to safely exercise the paid share/submission routes, while production does not expose it. Configure the documented, tightly restricted `SAFE_QA_FIXTURES=1` release-verification deployment/path, or change the release verifier to use a separately provisioned safe endpoint. Then rerun the command successfully against the deployed candidate. |

No claims test failed. The FAIL is specifically the fresh deployment-only
failure above; it supersedes the prior handoff's statement that no blockers
remain.
