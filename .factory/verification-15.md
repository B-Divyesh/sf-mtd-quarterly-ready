# Independent verification 15 — FAIL

**Verified:** 2026-08-29  
**Requested candidate:** `77e46dc1b7467b70f9cecd0ca2789eb221963185`  
**Live URL:** <https://mtd-quarterly-ready.sociobot.in>  
**Actual deployed build:** `77e46d4de4f96d28753dbf017a7d7067df737e0f`

## Release verdict: FAIL

Do not release this candidate. The requested SHA cannot be resolved from the
fresh clone or its `origin/main`, and live `/health` reports the different SHA
above. More importantly, fresh black-box checks show that the live backend
accepts a record save and immediately loses it on a subsequent read. The live
rate limiter is also split across three instances rather than enforcing the
documented single-client allowance. Production additionally has no approved
HMRC integration, so it does not meet the researched brief's end-to-end
submission requirement.

## Required first-read check — PASS

Cold-loading `/` on desktop produced status 200 with no browser errors. The
first screen says **“Turn records into a checked quarterly update”**, says it
is **“For UK sole traders, tutors and landlords … without a full accounting
suite”**, and makes **“Try it with sample data”** the first primary action.
Its adjacent explanation says it opens a private sample quarter with no
account. The action opens the `/demo` sandbox. This meets the plain-words and
one-click demo requirements.

## Claims gate — PASS locally from a fresh clone

Fresh clone: `/tmp/mtd-qa-OQJTgw`, checked out at
`77e46d4de4f96d28753dbf017a7d7067df737e0f`; `npm ci` installed 60 packages
with 0 vulnerabilities. `.factory/claims.json` exists and declares 22 claims.

- Every listed Playwright claim command was run against the local demo entry
  point. Consolidated evidence: `npx playwright test --grep @claim:` passed
  **13 tests** covering all 16 browser claim IDs.
- Every listed Rust claim command passed: HMRC reviewed approved-integration
  payload, accountant-link 30-day expiry, encrypted storage, hash-chained
  audit log, anonymous page count, and non-filing HMRC sandbox.

No missing or failing declared claim test blocked the local claims gate.

## Local quality checks — PASS (except unavailable Docker CLI)

| Check | Result |
| --- | --- |
| `npm test` | PASS: typecheck; 11 Vitest tests; 16 Rust tests; deployment-contract check; Vite production build; 47 Playwright tests. |
| `cargo fmt -- --check` | PASS. |
| `cargo clippy --all-targets -- -D warnings` | PASS. |
| `BUILD_SHA=77e46d4… cargo build --release` | PASS; release binary built. |
| Exact frontend production build | PASS: initial JS 46.24 KB / **14.99 KB gzip**; CSS 21.71 KB / **5.33 KB gzip**. |
| Container image build | NOT RUN: `docker` is not installed in this QA container. |

## Live verification

### Functional, accessibility, privacy and performance checks that passed

- The live browser suite ran 47 tests. **44 passed**, 3 failed as described
  below; passing coverage included desktop, 390px mobile, keyboard demo entry,
  visible focus, no horizontal overflow, 200% text, reduced motion, no console
  errors on cold records load, internal-link crawl, conditional submission
  copy, demo/offline/service-worker update, and the normal/invalid UI flows.
- Axe via `@axe-core/playwright` found **0 serious/critical** violations on
  `/`, `/demo`, `/privacy`, and `/terms`; the independent 390px `/demo` pass
  also found none. The repository has no `verify-url.sh`, so that named helper
  could not be run.
- The independent fresh `/demo` browser context made requests only to
  `https://mtd-quarterly-ready.sociobot.in` (document, local JS/CSS, and
  `/health`), stored **no cookies**, and logged **no console/page errors**.
  This supports the no-advertising-cookie/no-third-party-analytics claim for
  the demo flow.
- Security headers were present: CSP with `frame-ancestors 'none'`,
  `X-Content-Type-Options: nosniff`, strict-origin referrer policy,
  HSTS, and a restrictive permissions policy. Hashed JS/CSS use
  `public, max-age=31536000, immutable`; HTML and `sw.js` use `no-cache`.
  Unknown route returned a genuine 404.
- `/health` responds 200 and reports `safe_qa_fixtures: true`. The service
  worker registers, updates, controls the page, and the demo reloads offline
  after a first visit (claim test passed).

### Release-blocking findings

#### Critical — real records do not persist on the deployed backend

At 18:00 UTC, a fresh UUID workspace was `PUT` to the live
`/api/workspace` with a valid in-quarter transaction. The server replied
`200 {"saved":true,…}`. An immediate `GET` with the same workspace ID and
client header replied `200 {"document":null}`. A 12-workspace repetition
returned false for every read (**12/12**), and an additional single repeat
showed the same result. The live Playwright test
`workspace endpoints save and return an encrypted document` failed on the
same condition (`TypeError: Cannot read properties of null (reading
'transactions')`).

This violates the core job: a sole trader cannot safely keep a quarter if an
acknowledged save can disappear. It also contradicts the privacy/README
promise of encrypted server storage and requires a deployment correction,
not a frontend workaround.

#### Critical — per-client API rate allowance is not enforced as documented

The local source and deployment report promise 40 reads or 12 writes per
client. Fresh live sequential requests with one client identity observed:

- `POST /api/page-view`: **36** responses of 204; first `429` on request
  **37**, with `Retry-After: 1`.
- `GET /api/share/not-a-token`: **120** responses of 400; first `429` on
  request **121**, with `Retry-After: 1`.

This is exactly three times the documented 12/40 allowance, consistent with
three independent in-memory limiters. The live Playwright suite independently
failed its read-limit check (expected 429, got 400) and write-limit check
(expected 429, got 204). It also explains the missing workspace reads: writes
and reads are reaching separate, non-shared backend instances. The `429`
response and `Retry-After` header eventually exist, but the required allowance
is not enforced for a single client.

#### High — no approved HMRC integration in production

Live `/health` says:

```json
{"hmrc_integration_configured":false,"hmrc_integration_mode":"not_configured"}
```

`EXPECTED_BUILD_SHA=77e46d4… npm run verify:release` exits 1 with
`production has no approved HMRC integration configured`. The UI truthfully
hides direct submission and offers a reviewed handoff, but the researched
brief explicitly requires HMRC-compatible submission through an approved
integration. This deployment is a handoff-only partial product, not an
end-to-end release.

#### High — candidate identity is not deployable/verifiable

`git cat-file -t 77e46dc1b7467b70f9cecd0ca2789eb221963185` fails in both the
working checkout and a clean clone of the stated repository. Live `/health`
reports `77e46d4de4f96d28753dbf017a7d7067df737e0f`, which is the actual
current `origin/main` head and differs from the requested SHA. Consequently
the requested candidate cannot be shown to be deployed.

#### Medium — required accessibility helper is absent

No `verify-url.sh` exists anywhere in the clean clone. Equivalent manual and
Playwright/Axe checks passed, but the required worker helper cannot be run or
used as a repeatable release gate.

## Reproduction commands

```bash
# Candidate identity / live identity
git cat-file -t 77e46dc1b7467b70f9cecd0ca2789eb221963185
curl -sS https://mtd-quarterly-ready.sociobot.in/health

# Local claimed checks and complete local suite
npx playwright test --grep @claim:
cargo test claim_hmrc_submission_uses_an_approved_integration_after_human_review
cargo test claim_accountant_link_expiry
cargo test claim_encrypted_storage
cargo test claim_hash_chained_audit_log
cargo test claim_anonymous_page_count
cargo test claim_hmrc_sandbox_is_non_filing_and_sends_no_records_or_secret
npm test

# Live suite (reproduces the three failures)
VERIFY_ORIGIN=https://mtd-quarterly-ready.sociobot.in \
EXPECTED_BUILD_SHA=77e46d4de4f96d28753dbf017a7d7067df737e0f \
npx playwright test

# Approved-integration release gate
EXPECTED_BUILD_SHA=77e46d4de4f96d28753dbf017a7d7067df737e0f npm run verify:release
```

## Required next steps

1. Restore exactly one active backend replica or move SQLite persistence and
   rate-limit state to shared, safe infrastructure; prove read-after-write,
   restart durability, and the exact 40/12 allowance on the public ingress.
2. Deploy the actual intended commit and make its full SHA appear from
   `/health` before re-verification.
3. Provision and verify an HMRC-approved provider integration, with explicit
   human review and a safe authorised end-to-end test, before claiming the
   complete product is ready.
4. Add the missing `verify-url.sh` helper (or formally replace the mandated
   invocation in the product contract) and re-run independent QA.
