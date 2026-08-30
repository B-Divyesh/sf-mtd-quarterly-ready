# Independent verification 19 — FAIL

**Verified:** 2026-08-30  
**Candidate:** `4e94a89ab094ee886e3b0f19c7cd7720db1950a2`  
**Live URL:** <https://mtd-quarterly-ready.sociobot.in>

## Verdict: FAIL — do not release

The live service reports the requested source commit and its JS/CSS hashes
match a fresh local production build. The candidate nevertheless fails the
core product and backend contract. Live concurrent saves lose data; the
mandatory rate limit is intermittent; and production has no approved,
taxpayer-consented HMRC integration.

## Required first-read test — PASS

In a new cold Chromium context, `/` plainly said:

- **Does:** “Turn records into a checked quarterly update”.
- **For:** “UK sole traders, tutors and landlords who need MTD records without
  a full accounting suite.”
- **First click:** “Try it with sample data”, next to “Opens a private sample
  quarter. No account needed.”

The action opens `/demo` without an account. The cold page had title
`Quarterly Ready — Check your MTD quarter`, `lang="en-GB"`, one `h1`, one
`main`, no console/page errors, and no third-party requests.

## Release-blocking defects

### Critical — acknowledged concurrent saves are lost

Two independent 10-way live `PUT /api/workspace` probes used unique workspace
IDs and one unique, valid document per request. Every write returned **200**.
After all writes completed (and again after a 1.5 s wait), reading each ID
returned only **5/10** original documents; the other five returned
`{"document":null}` with HTTP 200. The second probe preserved IDs 0, 2, 6, 7,
and 9 only; IDs 1, 3, 4, 5, and 8 were absent.

This is unrecoverable loss of financial records after success responses. It
violates the backend persistence/concurrency requirement and makes the
quarterly record-to-handoff job unsafe. A 100-way `/health` smoke did return
100/100 HTTP 200 in 797 ms; the failure is specifically persistent workspace
writes.

### Critical — live rate limiting is not reliable

The documented allowance is 40 reads and 12 writes per client. A direct,
stable keep-alive probe did once pass: 429 on read 41 and write 13 with
`Retry-After: 58`. But the fresh full live verifier immediately reproduced the
failure: read request 41 returned **400**, not 429. The same error is produced
by the earlier release report and means rate limiting sometimes runs after
workspace validation, or is otherwise not consistently enforced. The backend
contract requires a reliable 429 plus positive `Retry-After` after the
allowance; intermittent compliance is a failure.

### Critical — no approved HMRC integration in production

Live `/health` returned:

```json
{
  "build_sha": "4e94a89ab094ee886e3b0f19c7cd7720db1950a2",
  "safe_qa_fixtures": true,
  "hmrc_integration_configured": false,
  "hmrc_integration_mode": "not_configured",
  "hmrc_taxpayer_consent_required": false,
  "hmrc_provider_name": null
}
```

`EXPECTED_BUILD_SHA=4e94a89ab094ee886e3b0f19c7cd7720db1950a2 npm run
verify:release` fails with `production has no approved HMRC integration
configured`. The handoff-only UI is honest, but it does not meet the brief's
approved-integration submission capability after explicit human review and
taxpayer consent.

## Passing evidence

- `.factory/claims.json` exists with 24 entries. From `npm ci`, every listed
  exact command was invoked individually against the configured local demo
  entry point. `npm test` then passed in full: TypeScript, 11 Vitest tests, 18
  Rust tests, deploy contract, production Vite build, and all 53 Playwright
  tests. This covers demo isolation/access, no third-party tracking, CSV and
  handoff downloads, review, CSV import and invalid-input recovery, receipts,
  real-quarter separation, consent behaviour, subscription gate, accountant
  link, offline reload, paid checkout, and the local rate-limit claim.
- `cargo fmt -- --check`, `cargo clippy --all-targets -- -D warnings`, and
  `BUILD_SHA=4e94a89ab094ee886e3b0f19c7cd7720db1950a2 cargo build --release`
  passed. The release binary also started with only `PORT=4190`, generated its
  encryption key, and returned the candidate SHA from `/health`.
- Fresh local build emitted JS **48.01 kB** (gzip **15.44 kB**) and CSS
  **21.71 kB** (gzip **5.33 kB**). Live asset SHA-256 values exactly matched:
  JS `a71c93ad134a0331731dcb0996d09dfc485d782a4576a822bb3d64b7487b17ac`,
  CSS `657cdda006edfa2359269c216a6bdd41bc10331d36e5e9d00675ee1529313afb`.
- Fresh desktop and 390 px/reduced-motion scans of `/`, `/demo`, `/privacy`,
  and `/terms` found no Axe serious/critical violations, no browser errors,
  exactly one `h1` and `main`, and no 390 px horizontal overflow. The visible
  focus outline was `rgb(11, 98, 92) solid 3px`. `npm run verify:url --
  https://mtd-quarterly-ready.sociobot.in/demo` passed.
- Demo privacy/PWA: fresh `/demo` had zero cookies and requests only to the
  product origin. It registered `/sw.js`, reloaded successfully offline with
  “Maya Patel Tutoring” visible, and emitted no errors. Asset responses have
  `public, max-age=31536000, immutable`; HTML has HSTS, nosniff, strict-origin
  referrer policy, permissions policy, and CSP including `frame-ancestors
  'none'`.
- End-to-end live demo: resolving the outstanding category, confirming review,
  downloading the CSV (19 rows), downloading a reviewed
  `quarterly-ready-mtd-itsa-handoff-v1` JSON handoff, and opening `/share/demo`
  all worked without browser errors.
- Live routes `/`, `/demo`, `/records`, `/privacy`, `/terms`, `/robots.txt`,
  and `/sitemap.xml` returned 200; a nonexistent route returned 404.
- No sign-in path is present, so the Entra External ID tenant condition is not
  applicable.

## Reproduce blockers

```sh
npm ci
# Run every command in .factory/claims.json individually
npm test
EXPECTED_BUILD_SHA=4e94a89ab094ee886e3b0f19c7cd7720db1950a2 npm run verify:live
EXPECTED_BUILD_SHA=4e94a89ab094ee886e3b0f19c7cd7720db1950a2 npm run verify:release
VERIFY_ORIGIN=https://mtd-quarterly-ready.sociobot.in node scripts/verify-rate-limit.mjs --kind read
VERIFY_ORIGIN=https://mtd-quarterly-ready.sociobot.in node scripts/verify-rate-limit.mjs --kind write
```

For persistence, concurrently write ten documents with distinct
`x-workspace-id` headers, then read each document back. Do not regard a 200
write response as durable until all ten survive. Fix transactional/concurrent
persistence and enforce the limiter before request validation, then provision
and verify a real approved HMRC provider and taxpayer OAuth consent flow.
