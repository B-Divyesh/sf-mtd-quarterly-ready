# Independent verification 2 — FAIL

Verified 2026-08-28 against candidate commit
`e7a37d14918ffb296268057892a370c7e5ac2305` and the live deployment
`https://mtd-quarterly-ready.sociobot.in`.

## Release decision

**FAIL.** The release is technically healthy, but it does not fulfil the
researched brief's required **HMRC-compatible submission through an approved
integration**. It deliberately exports a handoff JSON instead and explicitly
states that it does not submit to HMRC. That is a core job-to-be-done gap, not
a deployment-only failure. The paid accountant-link gate is also only a
client-side check: the unauthenticated backend share endpoint issues links
without a verified Sociobot licence.

## First read and deployment identity

Cold desktop load of `/` returned 200 with no console or page errors. In plain
words, the page says it turns records into a checked quarterly update, names
UK sole traders, tutors, and landlords as its users, and puts **Try it with
sample data** on the first screen. The adjacent text says it opens a private
sample quarter without an account. The link opens `/demo` in one click, with a
persistent **Demo — sample data, nothing is saved** banner. This gate passes.

`GET /health` returned:

```json
{"status":"ok","build_sha":"e7a37d14918ffb296268057892a370c7e5ac2305"}
```

The deployed JS and CSS have the same SHA-256 bytes as this checkout's built
`dist/assets/index-CIsi56uQ.js` and `dist/assets/index-DyVRUyEw.css`; the live
deployment therefore matches the candidate.

## Claims gate — pass

`.factory/claims.json` exists and declares 16 claims. From the installed clean
checkout, I ran every exact command recorded there (each browser grep was run
individually), followed by a consolidated run for captured evidence.

| Claims | Evidence | Result |
| --- | --- | --- |
| demo isolation/access, no tracking, no direct HMRC | tagged browser test | pass |
| accountant CSV, quarter review, CSV import, receipt capture | tagged browser tests | pass |
| HMRC handoff, accountant link, 30-day expiry | tagged browser tests | pass |
| encrypted storage, audit log, anonymous page count | the three exact `cargo test claim_…` commands | pass |
| offline browser copy, paid tier | tagged browser tests | pass |

The consolidated evidence was **10 Playwright tests passed** (the first test
carries four claim tags) and **3 Rust claim tests passed**. The browser claim
commands now build their demo entry point themselves; the earlier clean-
checkout harness failure is repaired.

## Local build and automated quality gates — pass

- `npm ci`: passed; 0 reported vulnerabilities.
- `npm test`: passed: TypeScript typecheck, 4 Vitest tests, 5 Rust tests,
  production Vite build, and 22 Playwright tests.
- `npm run build`: passed. Initial JS is 31.95 KB / **10.94 KB gzip** and CSS
  is 20.68 KB / **5.19 KB gzip**, comfortably within the 200 KB / 50 KB
  budgets. The largest shipped hero image is 51.7 KB.
- `cargo build --release`: passed; binary is 7.4 MB.
- `cargo clippy -- -D warnings`: passed.
- Container build could not be executed because this verifier image has no
  `docker` executable. The Dockerfile was inspected: it is multi-stage,
  non-root, uses `rust:1-alpine`, accepts the required build identity args,
  and defaults to `PORT=8080`.

## Product, accessibility, privacy, PWA, and backend exercise — pass

- Desktop and 390×844 live `/demo`: no horizontal overflow, console errors, or
  page errors. The 390px interface deliberately becomes labelled record slips.
- Live Axe: zero serious/critical findings on `/`, `/demo`, `/privacy`, and
  `/terms`; each has one `h1` and one `main`, with `lang=en-GB`.
- Keyboard: the sample action opened the demo with Enter. Its focused outline
  was a visible `rgb(11, 98, 92) solid 3px` with a 3px offset. Reduced-motion
  reduced the dial transition to `0.00001s`.
- End-to-end demo: resolving the outstanding category updated readiness;
  invalid CSV showed the stated missing-column recovery message; a £0 manual
  amount showed “The amount must be more than zero”; CSV export, receipt,
  review-gated handoff, and read-only demo link passed in the claim suite.
- Cold live request log contained only same-origin document, JS, CSS, hero
  image, and `POST /api/page-view`; the browser context had no cookies. No CDN
  scripts or fonts are used. Live CSP permits only self plus the documented
  Sociobot billing origin, and responses send `nosniff`, Referrer-Policy,
  Permissions-Policy, and `frame-ancestors 'none'`.
- Hashed live JS/CSS use `Cache-Control: public, max-age=31536000, immutable`.
- The live service worker was active and controlled the page (`quarterly-ready-v2`);
  after first visit, an offline live `/demo` reload showed both the offline
  banner and Maya Patel sample records.
- Release binary persistence: a workspace write survived a restart; its marker
  was absent from the SQLite file; invalid document shape returned 422 with a
  useful message. A 100-request concurrent `/health` smoke returned 100×200.
  With no application configuration set, the release binary listened on 8080,
  served health, and logged a generated encryption key (never the secret);
  on restart it logged persisted.
- Live per-client rate limit: 48 read requests returned **40×400 then 8×429**;
  16 write requests returned **12×204 then 4×429**. Both 429 responses sent
  `Retry-After: 1`. Observed allowances are 40 reads/sec and 12 writes/sec;
  `/health` is exempt.

## Defects

| Severity | Finding | Evidence and required resolution |
| --- | --- | --- |
| P0 | No HMRC-compatible submission via an approved integration | The brief names this as part of the smallest useful product. The landing page, README, and app instead say “No direct HMRC submission” and only download `quarterly-ready-mtd-itsa-handoff-v1` JSON. Implement and test an approved integration with explicit human review before submission, or obtain an explicitly revised acceptance contract. |
| P1 | Paid accountant links are not enforced server-side | `src/main.rs` exposes `POST /api/share` with only a workspace UUID and no licence verification. `tests/claims.spec.ts` successfully creates a live-style share using that endpoint without a licence. The browser's `isLicensed()` check can be bypassed by direct API use. Verify a Sociobot licence server-side before creating non-demo links. |
| P1 | Monetisation contradicts the researched brief | The brief specifies a subscription (£12/month or £99/year); the live page and claims instead sell a £99 **one-time** licence. Align the billing model and its copy/claims with the accepted brief, or revise the brief explicitly. |

No product code was changed during this verification. The only intended repository changes are this report and the verification addendum in `.factory/handoff.md`.
