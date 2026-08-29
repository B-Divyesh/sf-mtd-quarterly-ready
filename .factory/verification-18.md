# Independent verification 18 — FAIL

**Verified:** 2026-08-29
**Candidate:** `ca7cfc5e7f34547be6c9cd1963952952c2d28b82`
**Live URL:** <https://mtd-quarterly-ready.sociobot.in>

## Release verdict: FAIL

The live deployment identifies itself as the requested candidate, and its
hashed JavaScript and CSS exactly match a fresh local production build. It
cannot be accepted as a release because two central production requirements
fail: there is no approved HMRC direct-submission integration, and the live
API does not reliably enforce its documented rate limit.

## Required first-read test — PASS

A new, cold Chromium desktop context opened `/` and showed:

- **Does:** “Turn records into a checked quarterly update”.
- **For:** “UK sole traders, tutors and landlords who need MTD records without
  a full accounting suite.”
- **First action:** “Try it with sample data”, with “Opens a private sample
  quarter. No account needed.” beside it.

The one-click action was present and the page had title `Quarterly Ready —
Check your MTD quarter`, `lang="en-GB"`, one `h1`, and one `main`. The
browser reported no console or page errors.

## Claims gate — PASS locally

`.factory/claims.json` exists and has 23 entries. From the fresh `npm ci`
install I ran every exact `test` command listed in it individually against the
local configured demo entry point. Every command passed. This includes all
following claims:

| Claim IDs | Result |
| --- | --- |
| demo-isolation; demo-access; privacy-no-tracking; accountant-csv; quarter-review; free-quarter-persistence; csv-import; receipt-capture; receipt-locality; quarter-record-separation; hmrc-consent-no-records; conditional-submission; hmrc-handoff; accountant-link; server-licence-gate; offline-browser-copy; paid-tier | PASS — individual Playwright commands |
| hmrc-submission; accountant-link-expiry; encrypted-storage; audit-log; anonymous-page-count; hmrc-sandbox-no-filing | PASS — individual Cargo commands |

The manifest is complete and its demo route is `/demo`. A separate live
`tests/claims.spec.ts` run passed all 15 browser tests, including realistic
sample review, CSV import, invalid CSV recovery, receipts, review/handoff,
offline reload, and real-quarter persistence/separation.

## Release-blocking defects

### Critical — no approved HMRC MTD direct-submission integration

Live `/health` is the requested SHA but says:

```json
{
  "build_sha": "ca7cfc5e7f34547be6c9cd1963952952c2d28b82",
  "safe_qa_fixtures": true,
  "hmrc_integration_configured": false,
  "hmrc_integration_mode": "not_configured",
  "hmrc_taxpayer_consent_required": false,
  "hmrc_provider_name": null
}
```

The UI honestly offers a handoff rather than claiming a submission, but the
brief requires HMRC-compatible submission through an approved integration
after explicit human review. The product's own release gate fails exactly as
expected:

```text
EXPECTED_BUILD_SHA=ca7cfc5… npm run verify:release
Error: production has no approved HMRC integration configured
```

Provision an actual approved provider, OAuth taxpayer-consent registration,
and corresponding protected configuration; verify consent and a permitted
test submission before claiming release readiness.

### Critical — live API rate limit is not enforced after its stated allowance

The backend contract requires all server endpoints to return `429` and
`Retry-After` once a client exceeds its allowance. A first
`npm run verify:live` run reported 40 reads / 12 writes, but immediately
afterward fresh direct probes were consistently not limited:

- Three separate `VERIFY_ORIGIN=<live> node scripts/verify-rate-limit.mjs
  --kind read` runs: request 41 returned **400**, not 429.
- `--kind write`: request 13 returned **204**, not 429.
- An independent single keep-alive connection with fixed
  `X-Forwarded-For: 198.51.100.42` received 400 for each of the first 40
  workspace reads **and 400 again for request 41**. The socket was reused.

The 400 responses are normal missing-workspace-header validation responses;
they prove that the rate limiter did not run first. The disagreement with the
earlier passing release script means enforcement is not reliable in production,
which fails the mandatory server-side rate-limit acceptance condition.

## Evidence that passed

| Area | Fresh evidence |
| --- | --- |
| Live identity | `/health` reports the exact candidate SHA. Fresh local and live asset SHA-256 values matched: JS `a71c93ad…b17ac`, CSS `657cdda0…13afb`. |
| Persistence/concurrency | New 10-way live concurrent PUT then GET probe: 10/10 returned 200 and exactly restored their own document. The prior loss is not reproduced. |
| Local source checks | `npm ci`; TypeScript; 11 Vitest; 18 Rust unit tests; deploy contract; Vite production build; `cargo fmt -- --check`; `cargo clippy --all-targets -- -D warnings`; and `BUILD_SHA=<candidate> cargo build --release` passed. |
| Full-suite caveat | Two fresh `npm test` attempts did not complete green because bundled Chromium SIGSEGVed while creating a context; one disrupted run also timed out an ensuing dialog test. Both individually rerun local tests passed, and the live accessibility suite passed 20/20. This is a failed local command and is retained as evidence, not hidden. |
| Browser/accessibility | Live `tests/accessibility.spec.ts`: 20/20 passed, covering Axe serious/critical findings on `/`, `/demo`, `/privacy`, `/terms`; 390px layout; 200% text; target size; keyboard demo route; focusable dialog; link crawl; metadata; reduced motion; and no-error cold loads. `verify:url /demo` passed title/lang/main/h1/alt/console checks. |
| Privacy | A new demo browser context had zero cookies, zero errors, and only the product origin for document, JS, CSS, health, and favicon requests. No third-party analytics/tracking request occurred. |
| PWA/offline | The individual local offline claim and the fresh live claims suite passed the service-worker offline demo reload. |
| Headers/cache/budget | HTML has HSTS, nosniff, strict-origin referrer policy, permissions policy, CSP including `frame-ancestors 'none'`; JS and CSS have one-year immutable caching. Production build is 48.01 kB JS (15.44 kB gzip) and 21.71 kB CSS (5.33 kB gzip), below budget. |
| Routes and recovery | `/privacy`, `/terms`, `/records`, `/robots.txt`, `/sitemap.xml`, and designed unknown-route 404 respond correctly. Live invalid CSV recovery, receipt capture, accountant CSV/handoff, demo reset/isolation, and read-only demo pack are covered by the passing claims tests. |

No sign-in path is present, so the Microsoft Entra External ID requirement is
not applicable.

## Reproduction

```sh
npm ci
# run every exact .factory/claims.json test command individually
npm test
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
BUILD_SHA=ca7cfc5e7f34547be6c9cd1963952952c2d28b82 cargo build --release

EXPECTED_BUILD_SHA=ca7cfc5e7f34547be6c9cd1963952952c2d28b82 npm run verify:live
EXPECTED_BUILD_SHA=ca7cfc5e7f34547be6c9cd1963952952c2d28b82 npm run verify:release
VERIFY_ORIGIN=https://mtd-quarterly-ready.sociobot.in node scripts/verify-rate-limit.mjs --kind read
VERIFY_ORIGIN=https://mtd-quarterly-ready.sociobot.in node scripts/verify-rate-limit.mjs --kind write
```

Docker is unavailable in this verifier container, so an image build was not
run. Pre-existing `graphify-out/` worktree changes were preserved and excluded
from this verification.
