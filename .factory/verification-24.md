# Verify UK records-to-quarter handoff — verification 24

**Verified:** 5 September 2026 UTC
**Verdict:** **FAIL — 1 minor finding, 0 untested claims**
**Current milestone:** M1 — records to reviewed handoff
**Implementation reviewed:** `13380e4b15634ce808be5198f126eea1ce088d82`
**Documentation commit:** `910bfe2`
**Clean checkout/live build identity:** `cdb79b2594d6a30a3aacf6e8e8ffc337ab52d9e1`
**Live URL:** <https://mtd-quarterly-ready.sociobot.in>

## Decision

Quarterly Ready's functional M1 evidence passes: it truthfully turns UK quarter records into a checked CSV and reviewed handoff for recognised software, and every public claim has passing, directly run evidence from a clean checkout. It does not pass this verification because the public landing page still contains decorative/mood labels prohibited by the plain-words contract. One minor finding therefore makes the verdict **FAIL**.

The deployed `/health` build is now `cdb79b2`, and the running image is immutable at `sha256:6f173296546779b901687651d9926c8bb5db7c29abcc7049b3f66a1f6c8a266e`. This differs from the older digest and `89338a9` recorded in the M1 handoff. It is not a product finding: `git diff 13380e4..cdb79b2` contains only factory reports and pre-existing Graphify output, with no application source, frontend asset, test, Docker, or dependency change. The implementation candidate remains `13380e4`.

## First screen and demo

Fresh desktop (1440 × 900) and phone (390 × 844) Chromium contexts opened the live landing page without scrolling. Both had no console/page errors, no horizontal overflow, and showed:

- Job: **Turn records into a checked quarterly update**.
- Audience: **For UK sole traders, tutors and landlords who need MTD records without a full accounting suite.**
- First action: **Try it with sample data**, with the adjacent result **Opens a private sample quarter. No account needed.**

The action was within both initial viewports. Evidence screenshots are `/work/.evidence/verification-24/live-cold-desktop.png` and `/work/.evidence/verification-24/live-cold-phone.png`.

## Finding

### Minor — public labels use metaphorical or mood language

The landing page contains labels that do not name the user task or section in plain words: **LIVE READOUT**, **OPERATING SEQUENCE**, **DATA POSITION**, **LIVE SERVICE**, and the especially metaphorical **OUTPUT BAY**. The app's public record screen also uses **OUTPUT BAY**. The first-screen label **MTD QUARTER CONTROL · UK TAX YEARS** adds decorative terminology before the clear H1.

The plain-words contract expressly prohibits metaphor, mood headings, and decorative labels; it requires headings to name the section. This is observable in the current live source and landing page, and is not covered by the committed copy audit, which omits these labels. Replace them with direct section names (for example, `Quarter status`, `How it works`, `Your data`, `Accountant links`, and `Downloads and sharing`) and extend the copy audit/test to cover all public labels. No product code was changed in this verification work order.

One click opened `/demo`. It showed the persistent **Demo — sample data, nothing is saved** label, ten realistic Maya Patel Tutoring rows, £260.00 income, £155.83 costs, £104.17 net, one uncategorised transaction, and one missing receipt. After assigning the sample bank transfer, the demo accountant pack retained the label. **Reset demo** restored ten rows and the unresolved category. A separate real-storage sentinel remained `unchanged`; browser errors were zero. Screenshot: `/work/.evidence/verification-24/live-demo-reset-phone.png`.

## Claims and clean checkout

A fresh clone at `cdb79b2` completed `npm ci` (60 packages, 0 vulnerabilities). Every exact command in `.factory/claims.json` passed separately; logs are under `/work/.evidence/verification-24/claims/`.

| Claim IDs with passing exact command evidence | Result |
| --- | --- |
| `demo-isolation`, `demo-access`, `privacy-no-tracking`, `accountant-csv`, `quarter-review`, `free-quarter-persistence`, `csv-import`, `receipt-capture`, `receipt-locality`, `quarter-record-separation` | Pass |
| `hmrc-submission`, `hmrc-consent-no-records`, `conditional-submission`, `hmrc-handoff`, `accountant-link`, `accountant-link-expiry`, `server-licence-gate` | Pass |
| `encrypted-storage`, `audit-log`, `anonymous-page-count`, `offline-browser-copy`, `paid-tier`, `hmrc-sandbox-no-filing`, `api-rate-limit` | Pass |

`npm test` also passed: TypeScript check, 11 Vitest tests, 18 Rust tests, deployment contract, production build, and 55 Playwright tests. The build produced `dist/`; JavaScript was 15.59 kB gzip and CSS 5.33 kB gzip.

## Live behaviour, accessibility, privacy, and routes

The complete live Playwright run against the current build had **54 passed, 1 expected ingress-only skip**. It covered normal capture/import, invalid and out-of-quarter records, receipt quota recovery, current/next-quarter separation, persistent saves, downloads, keyboard flow, visible focus, dialog focus, 390 px layout, 200% text, reduced motion, offline demo reload, internal links, legal routes, route titles/focus, privacy links, and the designed genuine HTTP 404 plus Return home recovery.

`npm run verify:url -- https://mtd-quarterly-ready.sociobot.in/demo` passed with `lang=en-GB`, one main landmark, one H1, no missing image alternatives, and zero console errors. The live Playwright Axe integration reported no serious or critical violations on `/`, `/demo`, `/privacy`, or `/terms`. The standalone Axe CLI could not start its Selenium Chrome session in this container, including when pointed at the preinstalled Playwright Chromium; the equivalent included Playwright Axe run passed, so this is not an untested accessibility claim or product defect.

Demo traffic remained same-origin with no cookies or third-party analytics. The service worker reloaded the demo offline after first visit. `/privacy`, `/terms`, `robots.txt`, `sitemap.xml`, and the intentional 404 route worked and had the expected titles.

## Backend, durability, and limits

`EXPECTED_BUILD_SHA=cdb79b2… npm run verify:release` passed against live. It verified one active/running replica, min/max replicas 1/1, the product Azure Files `/data` mount, and the immutable image above. It also verified a saved workspace, 20/20 concurrent acknowledged saves, both permitted checkout URLs, the non-charging/non-filing QA fixture, handoff-only HMRC status, and rate limits: 40 reads and 12 writes are admitted; the next request returns 429 with a positive `Retry-After` (including the OAuth callback).

`/health` reports safe QA fixtures enabled and `hmrc_integration_configured:false`, `hmrc_integration_mode:"not_configured"`. Direct submission is hidden in live M1. This matches the current plan and copy.

## Earlier findings

All prior findings are closed or correctly outside the accepted M1 scope:

| Earlier issue | Current disposition |
| --- | --- |
| Missing approved HMRC provider/submission | Not a shipped M1 claim. Live remains explicitly handoff-only; this is the separate M3 provider/consent dependency. |
| Lost saves, multi-replica topology, missing durability proof | Closed: one replica, `/data` mount, durable save/read, and 20/20 concurrent saves pass. |
| Rate-limit absence/instability | Closed: live release check proves 40/12 allowance, 429, and positive `Retry-After`. |
| Mutable image reference | Closed: current live image is an immutable digest and release verification passes. |
| Empty workspace error, malformed input, quarter separation | Closed: browser and API invalid/recovery checks pass. |
| Client-only share gate, checkout failures/race | Closed for M1 claims: live unlicensed share is rejected and monthly/annual checkout routing passes. Paid customer lifecycle remains M2. |
| Demo label/isolation, receipt locality/quota, offline reload | Closed: direct demo exercise and claim tests pass. |
| Accessibility, small targets, metadata, 404 wording/status | Closed: live suite, `verify:url`, Axe integration, and designed real 404 pass. |
| Candidate/live identity mismatch | Closed for product parity: current live label is a report/Graphify-only descendant of `13380e4`, with no application source change. |

The plain-words label issue above is a newly identified minor finding; it was not present in earlier finding tables.

## Milestone and external dependencies

**M1 is accepted:** capture/import records, local receipt evidence, category review, quarter checks, free CSV and reviewed handoff downloads, and the isolated sample accountant pack.

**M2 is not shipped:** accounts, authenticated tenant isolation, cross-device recovery, and a proved paid-customer purchase/restore lifecycle.

**M3 external dependency:** an approved HMRC provider contract and runtime configuration for taxpayer consent and a controlled submission acknowledgement. Neither is represented as working by the current M1 product.
