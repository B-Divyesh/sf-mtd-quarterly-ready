# Independent verification 17 — FAIL

**Verified:** 2026-08-29
**Candidate / deployed build:** `4c20b33de43b97b1444a541314546159d67cc5d5`
**Live URL:** <https://mtd-quarterly-ready.sociobot.in>

## Release verdict: FAIL

The live application is the requested candidate, but it fails two release
requirements:

1. Production has no approved HMRC direct-submission integration.
2. Real workspace data does not reliably persist in the deployed backend.

Both are release blockers for a product whose central job is keeping compliant
quarterly records and producing/submitting the resulting update.

## Required first-read test — PASS

A cold desktop Chromium visit to `/` clearly said:

- **Does:** “Turn records into a checked quarterly update.”
- **For:** “UK sole traders, tutors and landlords who need MTD records without
  a full accounting suite.”
- **First click:** “Try it with sample data”; its adjacent text says it opens a
  private sample quarter without an account.

The action is present on the first screen and opens `/demo` in one click.
There were no console/page errors.

## Claims gate

`.factory/claims.json` exists and lists 22 claims. Before installing the clean
checkout's lockfile, the first listed command correctly could not load the
absent `@playwright/test` package. After `npm ci` (60 packages; 0
vulnerabilities), I ran **every exact command** listed in the file against the
local demo entry point:

- 16 individual Playwright claim commands: PASS.
- 6 individual Rust claim commands: PASS.

The full local `npm test` also passed (11 Vitest, 16 Rust, and 49 Playwright
tests). This satisfies the sandbox gate for the candidate source, but fresh
live runs invalidate the deployed persistence claims below.

## Release-blocking defects

### Critical — production lacks the approved HMRC integration required by the brief

Live `/health` returned:

```json
{
  "status": "ok",
  "build_sha": "4c20b33de43b97b1444a541314546159d67cc5d5",
  "safe_qa_fixtures": true,
  "hmrc_integration_configured": false,
  "hmrc_integration_mode": "not_configured"
}
```

The page honestly hides direct submission and offers a handoff instead, but the
researched acceptance contract calls for HMRC-compatible submission through an
approved integration after human review. The repository's own stricter release
gate independently fails:

```text
EXPECTED_BUILD_SHA=4c20b33de43b97b1444a541314546159d67cc5d5 npm run verify:release
Error: production has no approved HMRC integration configured
```

Provision and verify a genuine approved MTD provider, its taxpayer-consent
flow, and its credential before presenting this as a filing-capable release.

### Critical — deployed real workspaces acknowledge saves but lose records

The source's local claim passes, but its live equivalent fails reproducibly.
Running this against the deployed candidate three times failed all three times:

```sh
VERIFY_ORIGIN=https://mtd-quarterly-ready.sociobot.in \
  npx playwright test tests/claims.spec.ts \
  --grep '@claim:free-quarter-persistence' --repeat-each=3
```

Each failure received `200` for both `PUT /api/workspace` saves, then received
`{"document":null}` for at least one saved workspace. The failure is at
`tests/claims.spec.ts:98/99`, while reading the saved current/next-quarter
documents. The browser reload still shows local browser data, which masks the
server loss.

An independent 10-way live concurrency probe reinforced the result: all ten
distinct, valid workspace PUTs returned `200`; immediately reading each with
the same workspace ID returned only **1/10** matching documents. This violates
the claims that the free version keeps the working quarter and that each
quarter has separate browser **and server** records. It also makes the paid
accountant-link/data-retention workflow unsafe. Investigate the deployed
replica/storage/volume boundary; do not treat a 200 save response as durable
until it is read back after concurrent writes and a restart.

## Evidence that passed

| Area | Fresh evidence |
| --- | --- |
| Candidate identity and health | `EXPECTED_BUILD_SHA=4c20… npm run verify:live` passed; `/health` reports the exact candidate SHA. Its basic sequential durability probe passed, demonstrating the defect is missed by that probe. |
| Local checks | `npm test`, `cargo fmt -- --check`, `cargo clippy --all-targets -- -D warnings`, and `BUILD_SHA=4c20… cargo build --release` passed. |
| Production build budget | Vite build: JS 46.24 kB / 14.99 kB gzip; CSS 21.71 kB / 5.33 kB gzip. Live hashed JS/CSS use immutable one-year caching. |
| Demo and recovery | Live demo showed realistic tutor records, unresolved category/receipt states, CSV and handoff controls. Live invalid-CSV and receipt recovery regression paths passed before the persistence failure. |
| Accessibility | `VERIFY_ORIGIN=<live> npx playwright test tests/accessibility.spec.ts`: 19/19 passed, including Axe serious/critical checks on `/`, `/demo`, `/privacy`, and `/terms`, keyboard demo entry, focus, 390 px layout, 200% text, targets, and reduced motion. |
| Privacy | A fresh `/demo` request log contained only the product origin: document, hashed JS/CSS, and `/health`; no cookies, analytics, or third-party request occurred. Receipt/demo operations are covered by the passing local claims. |
| Headers/routing | Live responses include HSTS, `nosniff`, strict-origin referrer policy, restrictive permissions policy, and CSP with `frame-ancestors 'none'`. `/privacy`, `/terms`, `/records`, `/robots.txt`, `/sitemap.xml`, and designed `/404.html` returned 200; unknown application routes return 404. |
| PWA | The offline-browser-copy claim passed locally and in the live claim run: service worker `update()` returned active/controlled, then `/demo` reloaded offline with the sample data. |
| Rate limiting | Independent live `verify:live` observed 40 reads allowed then 429 on request 41 and 12 writes allowed then 429 on request 13, both with `Retry-After: 1`, over a single persistent client connection. |

No sign-in flow is required by this build, so the Entra tenant requirement is
not applicable.

## Reproduction

```sh
npm ci
# Run each command in .factory/claims.json individually
npm test
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
BUILD_SHA=4c20b33de43b97b1444a541314546159d67cc5d5 cargo build --release

EXPECTED_BUILD_SHA=4c20b33de43b97b1444a541314546159d67cc5d5 npm run verify:live
EXPECTED_BUILD_SHA=4c20b33de43b97b1444a541314546159d67cc5d5 npm run verify:release
VERIFY_ORIGIN=https://mtd-quarterly-ready.sociobot.in \
  npx playwright test tests/claims.spec.ts --grep '@claim:free-quarter-persistence' --repeat-each=3
```

Docker is not installed in this verifier container, so a local image build was
not possible. No product code was modified. Pre-existing `graphify-out/`
changes were preserved and excluded from this verification.
