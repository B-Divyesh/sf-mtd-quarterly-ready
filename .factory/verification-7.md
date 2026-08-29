# Quarterly Ready — independent verification 7

## Verdict: FAIL

Candidate `b26820a560ce27db2b7271dac0e204931c4c6888` is not ready for
release at `https://mtd-quarterly-ready.sociobot.in`.

Four independent release blockers were found:

1. The live `/health` response identifies build
   `5d1f989b266e2f320c172266f4ef0056977b4eba`, not the candidate.
2. The first claim command failed from the clean candidate checkout because
   the Playwright web-server timeout expired during its cold Rust build.
3. The real product is fixed to 6 April–5 July 2026. On 29 August 2026 there
   is no way to select or create the current or a later quarter.
4. CSV and API boundary validation can put invalid records into compliance
   totals and handoff data.

The candidate's executable source files and generated frontend assets are
byte-for-byte equivalent to the deployed build's source tree; only factory
reports/analysis differ between the candidate and live build commit. That
does not satisfy the explicit build-identity check.

## First-read and demo gate

**PASS.** In a fresh 1440×900 browser, the first screen says:

- what it does: “Turn records into a checked quarterly update”;
- who it is for: “UK sole traders, tutors and landlords” who need MTD records;
- what to do first: “Try it with sample data,” followed by “No account needed.”

The action opens `/demo` in one click. The resulting screen immediately shows
Maya Patel Tutoring, ten transactions, totals, missing checks, and a persistent
demo banner. No sign-in is required, so the Entra authority requirement is not
applicable.

## Mandatory claims gate

I created a detached clean worktree at the exact candidate, ran `npm ci`, then
ran all 17 `test` commands in `.factory/claims.json` separately and in listed
order. The browser tests used the repository's `/demo` entry point and a fresh
`/tmp/quarterly-ready-test` data directory for each command.

| Claim | Result | Evidence |
| --- | --- | --- |
| `demo-isolation` | **FAIL** | `npx playwright test --grep @claim:demo-isolation` timed out after 120,000 ms waiting for `config.webServer`; cold crate download/build was still compiling at 122.0 s. |
| `demo-access` | PASS | 1 Playwright test passed. |
| `privacy-no-tracking` | PASS | 1 Playwright test passed. |
| `accountant-csv` | PASS | 1 Playwright test passed. |
| `quarter-review` | PASS | 1 Playwright test passed. |
| `csv-import` | PASS | 1 Playwright test passed. |
| `receipt-capture` | PASS | 1 Playwright test passed. |
| `hmrc-submission` | PASS | 1 Rust test passed. |
| `hmrc-handoff` | PASS | 1 Playwright test passed. |
| `accountant-link` | PASS | 1 Playwright test passed. |
| `accountant-link-expiry` | PASS | 1 Rust test passed. |
| `server-licence-gate` | PASS | 1 Playwright test passed. |
| `encrypted-storage` | PASS | 1 Rust test passed. |
| `audit-log` | PASS | 1 Rust test passed. |
| `anonymous-page-count` | PASS | 1 Rust test passed. |
| `offline-browser-copy` | PASS | 1 Playwright test passed. |
| `paid-tier` | PASS | 1 Playwright test passed. |

Result: **16/17 commands passed; 1/17 failed.** The failed command passed in
5.6 s on a warm rerun, and the complete warm suite passed. That confirms the
feature works but does not erase the mandatory clean-checkout failure.

Cross-checking landing and README copy also found the unlisted claim “The free
version keeps your quarter” / “keeps one working quarter.” There is no claims
entry for free-quarter persistence, although non-claim integration tests do
cover workspace persistence. Under the claims contract, the copy must be
listed and tested or removed.

## Clean checkout, tests, and production build

Commands run at the exact candidate:

```sh
npm ci
npm test
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
BUILD_SHA=b26820a560ce27db2b7271dac0e204931c4c6888 cargo build --release
```

- `npm ci`: PASS; 60 packages installed and npm reported 0 vulnerabilities.
- `npm test`: PASS after the cold claims run warmed the Rust cache. It included
  TypeScript, 4 Vitest tests, 12 Rust tests, deploy-contract validation, the
  Vite production build, and 29 Playwright tests.
- `cargo fmt -- --check`: PASS.
- `cargo clippy --all-targets -- -D warnings`: PASS.
- Optimized Rust build: PASS.
- Frontend build: JS 36.38 kB raw / 11.93 kB gzip; CSS 21.22 kB raw /
  5.27 kB gzip; mobile hero 23.00 kB. All are within budget.
- Docker could not be built because this worker has no Docker daemon. The
  repository's deploy-contract test passed, and both production build stages
  were built directly.

## End-to-end product evidence

The live demo successfully completed the normal path:

- opened 10 sample transactions without an account;
- showed £260.00 income, £155.83 costs, and £104.17 net;
- rejected a manually entered zero amount with a useful error, then accepted
  the £0.01 boundary value;
- reported an invalid CSV header and recovered on the next valid import;
- rejected a 1,500,001-byte receipt, then accepted a small PDF;
- resolved the sample category, confirmed human review, downloaded the CSV
  and HMRC handoff, and opened the read-only sample accountant pack;
- the handoff had the stated format, period, reviewed flag, and correct totals.

Demo isolation passed independently: only
`demo:quarterly-ready:document` existed, cookies were empty, and every request
during the flow stayed on the product origin. “Start for real” removed the
demo key and opened an empty real quarter with separate real/workspace keys.

Invalid-data probes exposed unsafe recovery paths:

- CSV date `2026-07-06`, one day beyond the fixed quarter, was accepted and
  increased displayed quarter income from £260 to £360.
- Impossible date `2026-02-30` was accepted and displayed as 2 March while
  still increasing the quarter total.
- A zero-value CSV row was accepted.
- Category `Bananas` was stored unchanged but displayed as `Sales`. After the
  original unresolved row was fixed, the UI said every transaction had a
  category, enabled HMRC handoff, and exported `"bananas": 25` under
  `periodExpenses`.
- The candidate API correctly rejects impossible transaction dates and values
  above £1,000,000, but accepts an out-of-quarter transaction and accepts
  `quarterStart: "2026-02-30"`, both with HTTP 200.

The real app has no quarter selector or create-next-quarter action. Its empty
document, heading, form limits, exports, and API fixtures are all hard-coded to
6 April–5 July 2026. This period had already ended on the verification date.

## Deployment, backend, billing, and limits

```text
GET /health
200 {"status":"ok","build_sha":"5d1f989b266e2f320c172266f4ef0056977b4eba"}
```

`EXPECTED_BUILD_SHA=b26820a... npm run verify:live` failed immediately with
the same mismatch. A verifier run without the expected SHA passed the deployed
build's monthly and annual checkout creation, durable workspace round trip,
malformed-transaction rejection, 404 behavior, and rate limits. No purchase
was completed.

The candidate and live frontend `index.html`, JS, CSS, and hero asset had
identical SHA-256 hashes. `git diff` found no executable-source changes between
candidate `b26820a...` and live build commit `5d1f989...`; the build identity is
nevertheless not the requested candidate identity.

Candidate backend checks:

- startup with only `PORT` succeeded and logged build SHA, generated/persisted
  key state, and integration state without exposing secrets;
- 8 concurrent writes succeeded and a read returned the expected record;
- the record survived a graceful restart;
- raw SQLite files did not contain the plaintext test record;
- 5 MiB-plus input was rejected;
- live allowance: 40 reads/s and 12 writes/s, followed by 429 with
  `Retry-After: 1`; `/health` is exempt;
- Sociobot licence verification allowance: 30 invalid-token reads were
  accepted, then 8 returned 429 with `Retry-After: 4`.

No authorised subscription token was available. Therefore the live paid
accountant-link and approved-integration submission could not be proven end to
end; only checkout creation, denial without entitlement, and mock integration
tests were verified.

## Privacy, accessibility, PWA, headers, and performance

- Live request log: same-origin only throughout the demo; no cookies,
  advertising, analytics, console errors, page errors, or failed responses.
- Responses include CSP with `frame-ancestors 'none'`, nosniff,
  `Referrer-Policy`, and restrictive `Permissions-Policy`. HSTS was not present.
- Hashed assets use `Cache-Control: public, max-age=31536000, immutable`.
  HTML and `sw.js` have `Last-Modified` but no explicit `Cache-Control`.
- Service worker registration and `update()` succeeded. Offline reload after
  the first visit retained the sample and showed “Offline — browser copy
  active.” The site ships a manifest but does not link it from HTML, so it was
  assessed as a web app with offline support, not an installable PWA.
- Axe found 0 serious/critical issues on the live landing and completed demo.
- Keyboard-only demo activation, route focus, browser back navigation, and
  the 3 px visible focus ring passed.
- Normal 390 px layout had no horizontal overflow. At 200% text on a 1280 px
  viewport there was no horizontal overflow. Reduced motion reduced animation
  and transition duration to 0.01 ms.
- One mobile target misses the 44 px baseline: the “I checked these figures”
  label is 40.8 px high. Hidden file inputs are backed by 44 px labels.
- All tested routes have `lang="en-GB"`, one `main`, one `h1`, and route-specific
  runtime titles. Canonical and social metadata remain fixed to `/` on every
  SPA route.
- Lighthouse mobile: performance 100, accessibility 100, best practices 100,
  SEO 100; FCP 1.2 s, LCP 1.4 s, TBT 40 ms, CLS 0. Lighthouse noted absent
  text compression, about 11 KiB unused CSS, and about 26 KiB unused JS, but
  measured budgets still passed.

## Defects by severity

### P0 — release blockers

1. **Candidate identity is not deployed.** Deploy the candidate and require
   `/health` to return the exact candidate SHA before verification.
2. **A mandatory claim command fails in a cold checkout.** Increase the
   Playwright web-server startup timeout or prebuild/fetch through the declared
   command so every claim passes without a warm cache.
3. **The app cannot serve the current or subsequent quarter.** Add explicit
   UK quarter selection/creation, storage separation, rollover, and tests. Do
   not silently lock real users to the expired Q1 2026–27 period.
4. **Invalid CSV/API data can corrupt a quarter and HMRC handoff.** Validate
   real calendar dates, period membership, positive bounded amounts, and the
   category allow-list before mutating browser or server data. Show row-level
   errors and allow a corrected import.
5. **An unlisted persistence claim appears in landing/README copy.** Add a
   clean-state claim test for free-quarter persistence or remove/rephrase the
   claim.

### P1 — high

1. **The demo accountant page loses sandbox identity.** `/share/demo` displays
   sample data without the required persistent demo banner, Reset demo, or
   Start for real controls. A visitor can mistake it for a real shared pack.
2. **Core paid outcomes remain unverified in production.** Supply a safe test
   entitlement and approved-integration fixture, then prove the live
   accountant link and submission reference without making a real tax filing.

### P2 — medium/low

1. Increase the mobile review-checkbox label from 40.8 px to at least 44 px.
2. Update canonical/Open Graph metadata per route instead of leaving every
   route canonicalized to the landing page.
3. If installability is intended, link `manifest.webmanifest` and provide
   conforming install icons; otherwise remove the unused manifest.

## Required re-verification

After repair and deployment, repeat every claim from a cold dependency/cache
state, verify the exact `/health` SHA, exercise current and next quarters,
repeat all invalid CSV cases, and run the live paid link/submission path with a
non-production entitlement and integration fixture.
