# Quarterly Ready — strict review 1

**Reviewed:** 5 September 2026 UTC
**Verdict:** **PASS — 0 findings, 0 untested claims**
**Current milestone:** M1 — records to reviewed handoff
**Live URL:** <https://mtd-quarterly-ready.sociobot.in>

## Review identity

- **Implementation candidate reviewed:** `575aa8659469ab6f45bce623c3da560a82248895`
- **Prior verification-report documentation commit:** `8b7b017fad59364c0e1886378b7164d446e29a59`
- **Current clean-checkout/Graphify-only documentation commit:** `0b6baad95205a6f22046aa2a8b003f7addb37c60`
- **Live health build label:** `73ddca8e56958abbdccff19a140d505f6bd8527a`
- **Live immutable image:** `sociobotregistry.azurecr.io/sf-mtd-quarterly-ready@sha256:06a13cf9ca86518490bcc5ac4257b63d97fe4a7fa05b744f22753442746a316e`

The live label is a descendant of the implementation candidate. Its diff
against `575aa865` contains the handoff, Graphify output, and an
accessibility regression test only; it contains no runtime frontend, Rust,
asset, dependency, Docker, or build configuration change. The reviewed
runtime implementation is therefore `575aa865`; the distinct documentation
and live-label SHAs are recorded above.

## Decision

**PASS.** There are no findings at any severity and no untested declared
claims. The earlier plain-words finding is closed: public labels name the
task or result, rather than using decorative or mood language. The current
M1 remains an honest records-to-reviewed-handoff product, not an MTD filing
service.

## Cold live experience and demo sandbox

Fresh Chromium contexts opened the live landing page at desktop (1440 × 900)
and phone (390 × 844) widths without scrolling. In both:

- title: `Quarterly Ready — Check your MTD quarter`;
- job/H1: **Turn records into a checked quarterly update**;
- audience: UK sole traders, tutors, and landlords;
- first action: **Try it with sample data**, visible in the initial viewport;
- no horizontal overflow and no console or page errors.

One click entered `/demo`. Both contexts showed the persistent **Demo —
sample data, nothing is saved** label, **Maya Patel Tutoring**, ten realistic
record rows, **Reset demo**, and **Start for real**. Reset restored the
ten-row sample and retained the persistent demo label. This exercise did not
use a real workspace.

Verification 24's minor finding named `LIVE READOUT`, `OPERATING
SEQUENCE`, `DATA POSITION`, `LIVE SERVICE`, `OUTPUT BAY`, and the
first-screen decorative label as prohibited language. Current public source
and live UI use direct replacements including **Quarter status**, **How it
works**, **What this tool does not do**, **Your data**, **Review your
quarter**, and **Downloads and sharing**. The committed copy audit now
includes those public labels. The finding is closed.

## Declared claims and clean verification

`npm ci` completed with 60 packages and zero reported vulnerabilities. Every
one of the 24 exact commands from `.factory/claims.json` completed
successfully from this clean setup:

| Claim ID | Exact declared command | Result |
| --- | --- | --- |
| demo-isolation | `npx playwright test --grep @claim:demo-isolation` | Pass |
| demo-access | `npx playwright test --grep @claim:demo-access` | Pass |
| privacy-no-tracking | `npx playwright test --grep @claim:privacy-no-tracking` | Pass |
| accountant-csv | `npx playwright test --grep @claim:accountant-csv` | Pass |
| quarter-review | `npx playwright test --grep @claim:quarter-review` | Pass |
| free-quarter-persistence | `npx playwright test --grep @claim:free-quarter-persistence` | Pass |
| csv-import | `npx playwright test --grep @claim:csv-import` | Pass |
| receipt-capture | `npx playwright test --grep @claim:receipt-capture` | Pass |
| receipt-locality | `npx playwright test --grep @claim:receipt-locality` | Pass |
| quarter-record-separation | `npx playwright test --grep @claim:quarter-record-separation` | Pass |
| hmrc-submission | `cargo test claim_hmrc_submission_uses_an_approved_integration_after_human_review` | Pass (configured mock only) |
| hmrc-consent-no-records | `npx playwright test --grep @claim:hmrc-consent-no-records` | Pass (configured local service) |
| conditional-submission | `npx playwright test --grep @claim:conditional-submission` | Pass (configured local service) |
| hmrc-handoff | `npx playwright test --grep @claim:hmrc-handoff` | Pass |
| accountant-link | `npx playwright test --grep @claim:accountant-link` | Pass |
| accountant-link-expiry | `cargo test claim_accountant_link_expiry` | Pass |
| server-licence-gate | `npx playwright test --grep @claim:server-licence-gate` | Pass |
| encrypted-storage | `cargo test claim_encrypted_storage` | Pass |
| audit-log | `cargo test claim_hash_chained_audit_log` | Pass |
| anonymous-page-count | `cargo test claim_anonymous_page_count` | Pass |
| offline-browser-copy | `npx playwright test --grep @claim:offline-browser-copy` | Pass |
| paid-tier | `npx playwright test --grep @claim:paid-tier` | Pass |
| hmrc-sandbox-no-filing | `cargo test claim_hmrc_sandbox_is_non_filing_and_sends_no_records_or_secret` | Pass (non-filing mock) |
| api-rate-limit | `npx playwright test --grep @claim:api-rate-limit` | Pass |

`npm test` completed successfully: TypeScript check; 11/11 Vitest tests;
18/18 Rust tests; deployment-contract check; production build; and 56/56
Playwright tests. The production bundle measured 15.32 kB gzip JavaScript and
5.32 kB gzip CSS.

The configured-provider tests are conditional capability tests, not evidence
that M3 is live. The live health response confirms
`hmrc_integration_configured:false` and
`hmrc_integration_mode:"not_configured"`; direct submission is consequently
hidden. This is truthful for M1 and is not a finding.

## Accessibility, routes, privacy, and recovery

- `npm run verify:url -- <live route>` passed for `/`, `/demo`,
  `/privacy`, and `/terms`: correct route titles, `lang=en-GB`, exactly
  one `main` and H1, no missing image alternatives, and zero console errors.
- Fresh live Axe checks on those four routes found **0 violations**, including
  **0 serious or critical** violations.
- The full browser suite passed keyboard demo entry, focus handling, 44 px
  mobile targets, 200% text, reduced motion, offline demo reload, link crawl,
  legal routes, and designed 404 recovery.
- A direct unknown live route returned the expected HTTP **404** and the
  designed page offers **Return home**. This is correct recovery, not a
  defect.
- The demo claim continues to prove separate `demo:` storage, no cookies,
  and same-origin traffic only. Receipt and real-quarter isolation, invalid
  input, boundary, and recovery paths are exercised by the declared claims
  and full suite.

## Backend and live release

`EXPECTED_BUILD_SHA=73ddca8e56958abbdccff19a140d505f6bd8527a npm run
verify:release` passed. It verified:

- immutable image deployment, single active revision, and one min/max/running
  replica;
- the product-owned Azure Files mount at `/data`;
- durable workspace save/read plus 20 acknowledged concurrent documents;
- monthly and annual Sociobot checkout routing and the safe non-charging,
  non-filing fixture;
- M1 handoff-only HMRC status; and
- 40 reads and 12 writes admitted per stable client before 429, with positive
  `Retry-After`, including the OAuth callback at write 13.

No deployment, provider, billing, storage, or replica setting was changed by
this review.

## Earlier finding disposition and scope

| Earlier review issue | Disposition |
| --- | --- |
| Verification 24 decorative/mood labels | Closed; live labels and cold-screen regression are plain task/result names. |
| 404 wording/status | Closed; real HTTP 404 has plain copy and a working recovery path. |
| Durable persistence, replica count, immutable deployment, concurrent saves | Closed; live release verifier passed. |
| Rate-limit enforcement | Closed; live 40/12 allowance and 429/`Retry-After` proof passed. |
| Demo isolation, receipt locality, offline, invalid/recovery, accessibility, metadata, legal routes | Closed; declared claims and full suite passed. |
| Checkout and unlicensed live-link gate | Closed for M1 claims; this is not proof of a paid-customer lifecycle. |
| Approved provider, taxpayer consent, submission | Not an M1 defect and not shipped; a separate M3 dependency remains. |

## Current milestone and external dependencies

**M1 is the current shipped milestone.** It lets the intended UK sole trader,
tutor, or landlord capture/import quarter records, keep receipt files in the
browser, resolve review checks, and download CSV or a reviewed handoff. It
does not file a tax return.

**M2 is not shipped.** Accounts, authenticated tenant isolation, cross-device
recovery, and a paid purchase/restore lifecycle remain future work. Its
external dependencies are Sociobot Entra CIAM and the authorised Sociobot
billing entitlement flow.

**M3 is not shipped.** Approved-provider consent and controlled MTD submission
remain future work. Its external dependencies are an approved provider
contract, runtime configuration, taxpayer consent, and a controlled
provider/HMRC acknowledgement. The live service is correctly unconfigured
and makes no contrary public promise.
