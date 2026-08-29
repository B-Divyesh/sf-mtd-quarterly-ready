# Independent verification 16 — FAIL

**Verified:** 2026-08-29  
**Candidate / live build:** `9c0c66f8427e28503d4ed8789a8de7496f9efc3f`  
**Live URL:** <https://mtd-quarterly-ready.sociobot.in>

## Release verdict: FAIL

The deployed page and the candidate identity are healthy, but this is not an
end-to-end MTD release. Production `/health` reports
`"hmrc_integration_configured": false` and `"hmrc_integration_mode":
"not_configured"`. Consequently the UI hides direct submission and offers
only a reviewed handoff. The researched brief and product contract require a
reviewed quarterly submission through an approved HMRC integration. This
missing production capability is release-blocking.

## Required first-read check — PASS

A cold desktop load of `/` gave this plain reading:

- **Does:** “Turn records into a checked quarterly update.”
- **For:** UK sole traders, tutors and landlords who need MTD records without
  a full accounting suite.
- **First click:** “Try it with sample data”; adjacent text says it opens a
  private sample quarter with no account needed.

The one-click action is present and opens `/demo`. The landing load made only
same-origin requests (`/`, hashed JS/CSS, `/api/page-view`, `/health`, and its
local image) and had no console or page errors.

## Claims gate — PASS

`.factory/claims.json` is present and declares 22 claims. From the clean
candidate checkout after `npm ci` (60 packages, 0 vulnerabilities), I ran each
listed command through the local demo entry point. The aggregate browser run
`npx playwright test --grep @claim:` passed **13 tests**, covering all browser
claim IDs; `npm test` passed all 49 tests. The six declared Rust-claim commands
passed within the 16-test Rust suite.

This covers demo isolation/access/privacy, CSV import/export, quarterly
review/persistence/separation, receipt locality and quota recovery, conditional
submission, handoff/link/licence behaviour, encrypted storage, audit chain,
anonymous counting, offline reload, checkout, and the non-filing sandbox.

## Local quality gates — PASS

| Check | Evidence |
| --- | --- |
| `npm test` | PASS — typecheck, 11 Vitest tests, 16 Rust tests, deploy-contract, Vite build, 49 Playwright tests. |
| `cargo fmt -- --check` | PASS |
| `cargo clippy --all-targets -- -D warnings` | PASS |
| `BUILD_SHA=9c0c66f… cargo build --release` | PASS |
| Production Vite build | PASS — JS 46.24 kB (14.99 kB gzip); CSS 21.71 kB (5.33 kB gzip). |
| Container build | Not run: Docker CLI is unavailable in this QA container. Dockerfile was inspected; it is multi-stage, non-root, accepts the required build args, and uses `rust:1-alpine`. |

## Live checks — PASS except the release blocker

- `/health` returned the exact candidate SHA and `safe_qa_fixtures: true`.
  `EXPECTED_BUILD_SHA=9c0c66f… npm run verify:live` passed: durable
  save/read, malformed input rejection, checkout boundaries, safe fixture and
  rate limits.
- `npm run verify:url -- https://mtd-quarterly-ready.sociobot.in/demo` passed:
  title, `lang=en-GB`, one main landmark, one H1, alt text and no browser
  errors.
- Desktop and 390 px checks passed. A 390 px cold `/demo` context had no
  horizontal overflow (`390/390`), no cookies, no console/page errors, and only
  same-origin requests. The repository's local suite passed its keyboard,
  focus, 200% text, touch-target, mobile, reduced-motion and Axe baseline
  checks; the live browser run passed Axe serious/critical checks on `/`,
  `/demo`, `/privacy`, and `/terms` before endpoint-focused checks.
- Privacy/headers: no third-party request or cookie was observed in the demo.
  Live HTML has CSP including `frame-ancestors 'none'`, `nosniff`, strict-origin
  referrer policy, restrictive permissions policy and HSTS. HTML, 404 and
  `sw.js` are `no-cache`; hashed JS is
  `public, max-age=31536000, immutable`. Unknown routes return HTTP 404.
- PWA: the local claim test proved service-worker-controlled offline demo
  reload after first visit.
- Backend allowance observed from one persistent client: 40 read requests then
  429 on request 41, and 12 writes then 429 on request 13; each 429 included
  `Retry-After: 1`.

### Note on the live Playwright transport check

With `VERIFY_ORIGIN` set, the repository's two rate-limit regression tests
failed under Playwright's HTTP/1.1 request context (request 41 remained 400 and
write 13 remained 204), even though the trace shows the configured test header.
The same tests pass locally and an independent persistent-client live probe
observed the documented 40/12 limit and `Retry-After`. This is recorded as a
test-harness/ingress-connection discrepancy, not evidence that the documented
single-client allowance is absent. A targeted live rerun with the expected
build SHA passed health and workspace persistence; 12 fresh UUID save/read
probes also passed.

## Defects

### Critical — approved HMRC submission is not configured in production

Fresh evidence:

```json
{
  "status": "ok",
  "build_sha": "9c0c66f8427e28503d4ed8789a8de7496f9efc3f",
  "safe_qa_fixtures": true,
  "hmrc_integration_configured": false,
  "hmrc_integration_mode": "not_configured"
}
```

`EXPECTED_BUILD_SHA=9c0c66f… npm run verify:release` exits 1 with
`production has no approved HMRC integration configured`. This prevents the
core reviewed quarterly submission required by the brief. Configure a genuine
approved provider with authorised taxpayer consent and run the release verifier
again. Do not describe the handoff-only deployment as a completed MTD
submission product.

### Medium — live rate-limit automated regression is not transport-stable

The production endpoint policy works under a persistent single-client probe,
but the checked-in Playwright live regression cannot observe it through this
environment's HTTP/1.1 ingress path. Make that live test use a single stable
client connection or an ingress-supported client identity so it remains a
reliable deployment gate.

## Reproduction

```sh
npm ci
npm test
npx playwright test --grep @claim:
EXPECTED_BUILD_SHA=9c0c66f8427e28503d4ed8789a8de7496f9efc3f npm run verify:live
EXPECTED_BUILD_SHA=9c0c66f8427e28503d4ed8789a8de7496f9efc3f npm run verify:release
npm run verify:url -- https://mtd-quarterly-ready.sociobot.in/demo
curl -sS https://mtd-quarterly-ready.sociobot.in/health
```
