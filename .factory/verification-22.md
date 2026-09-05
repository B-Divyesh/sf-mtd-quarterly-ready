# Verify records-to-quarter handoff — verification 22

**Verified:** 5 September 2026 UTC

**Verdict:** **FAIL — 2 findings, 0 untested claims**

**Isolation/deployment candidate:** `3e34f53e0d4b5ce13d058a3d2dd6f238388c088c`

**Documentation checkout:** `1c893635f66aeb48eec3270dee7cdf25993ceb52`

**Live runtime build:** `7c840e4853bbcb16270977bdb568271ebd86c746`

**Last application-code change in the live lineage:** `a428876efd57704a2617e67e983ffe561b6abee0`

**Live URL:** <https://mtd-quarterly-ready.sociobot.in>

## Decision

The scoped isolation cleanup passes. The deployment contract is fail-closed,
accepts only `handoff-only`, contains no shared-vault lookup or HMRC secret
reference, and requires the live service to report `not_configured`. The live
topology, persistence, request limits, demo, accessibility, and all 24 declared
claims also pass.

The product still fails the researched product contract. It cannot submit a
reviewed quarterly update through an approved HMRC integration. It correctly
offers a handoff instead, but that does not complete the required job. The
designed 404 also uses a metaphor and a mood label, contrary to the supplied
plain-words contract. A deliberate HTTP 404 is expected; the response status is
not the defect.

## Findings

### Critical — the required live HMRC submission is unavailable

Live `/health` reports:

```json
{
  "hmrc_integration_configured": false,
  "hmrc_integration_mode": "not_configured",
  "hmrc_taxpayer_consent_required": false,
  "hmrc_provider_name": null
}
```

The records page says that no approved direct-submission integration is
configured and offers a reviewed handoff for recognised software. The safe QA
submission is explicitly non-filing. This is honest and is the required state
for the isolation repair, but the researched smallest useful product requires
submission through an approved integration after taxpayer consent and human
review.

Resolution requires a separately authorised, product-owned approved-provider
contract and consent flow. It must not restore a shared-vault dependency.

### Minor — the 404 recovery copy breaks the plain-words contract

An unknown route correctly returns HTTP 404 with a route-specific title, one
H1, one main landmark, and a link home. Its visible label is `NO SIGNAL` and its
H1 is `This page is not on the panel`. Those are mood/metaphor phrases rather
than plain error text. Replace them with direct wording such as `Page not found`
and keep the existing recovery link.

## First screen and one-click sample

Fresh 1440 × 900 desktop and 390 × 844 phone contexts were opened without
scrolling. Both showed:

- Job: `Turn records into a checked quarterly update`.
- Audience: UK sole traders, tutors, and landlords who need MTD records without
  a full accounting suite.
- First action: `Try it with sample data`, beside `Opens a private sample
  quarter. No account needed.`

The phone document width was 390 px with no horizontal overflow. The action
opened `/demo` in one click. The persistent banner said `Demo — sample data,
nothing is saved` and exposed `Reset demo` and `Start for real`.

The Maya Patel Tutoring sample contained ten realistic transactions for 6 April
to 5 July 2026. It showed £260.00 income, £155.83 costs, £104.17 net, one missing
category, and one missing receipt.

In one continuous live flow:

- The outstanding bank transfer was assigned to Sales and the figures were
  confirmed.
- The accountant CSV downloaded with 19 CRLF lines and all sample records.
- The HMRC handoff used `quarterly-ready-mtd-itsa-handoff-v1`, covered
  2026-04-06 through 2026-07-05, reported £260 turnover, and set
  `reviewedByUser:true`.
- The read-only sample link resolved to `/share/demo`.
- The demo banner remained visible after changes.
- `Reset demo` restored ten rows, the unresolved category, and the unchecked
  review state.
- A real-data sentinel remained unchanged, demo actions made no workspace API
  request, and `Start for real` removed all `demo:` keys before opening an empty
  real quarter.

Screenshots are stored outside the repository at
`/work/.evidence/verify22-live-cold-desktop.png`,
`/work/.evidence/verify22-live-cold-phone.png`, and
`/work/.evidence/verify22-live-demo-output-desktop.png`.

## Claims and clean-checkout gates

A fresh clone at documentation SHA `1c89363` was used. After the documented
`npm ci` prerequisite, every exact command in `.factory/claims.json` was run
separately. Result: **24 passed, 0 failed, 0 untested**.

The 18 browser claims passed for demo isolation/access, no tracking, CSV output,
quarter review, real-quarter persistence and separation, CSV import, receipt
capture/locality, consent privacy, conditional submission, handoff, accountant
link, licence gate, offline reload, paid checkout, and API limits. The six Rust
claims passed for reviewed provider submission logic, link expiry, encrypted
storage, audit chaining, anonymous page count, and non-filing sandbox behavior.

The landing page, app, privacy and terms pages, README, demo guide, and manifest
were cross-checked. No declared claim is missing a test, and no material public
capability or privacy claim was left untested. The approved-provider tests use
fixtures and prove conditional code behavior; they do not prove a provider is
configured in production, which is the critical finding above.

Clean-checkout results:

| Check | Result |
| --- | --- |
| `npm ci` | Pass; 60 packages, 0 vulnerabilities |
| `npm test` | Pass; typecheck, 11 Vitest, 18 Rust, deployment contract, Vite build, 54 Playwright |
| `npm run build` | Pass; `dist/` produced |
| `cargo fmt --all -- --check` | Pass |
| `cargo clippy --all-targets --all-features -- -D warnings` | Pass |
| `npm audit --audit-level=high` | Pass; 0 vulnerabilities |
| release Rust build | Pass with the current stable toolchain |

The production build is 48.37 kB JavaScript (15.59 kB gzip) and 21.71 kB CSS
(5.33 kB gzip), within budget.

## Live browser, accessibility, privacy, and routes

The full live browser suite finished with **53 passed and 1 expected non-claim
skip**. The skipped direct-origin fallback is inapplicable behind the live
ingress; every claim test ran. Independent `verify:url` on `/demo` passed with
`lang="en-GB"`, title, one H1, one main landmark, complete alt text, and no
console errors.

The suite and fresh browser checks covered keyboard-only demo entry, dialog
focus, visible focus, 44 px targets, 200% text, 390 px layout, reduced motion,
route focus/titles, internal links, cold empty state, invalid and boundary
inputs, receipt quota recovery, and Axe on `/`, `/demo`, `/privacy`, and
`/terms`. Axe found no serious or critical issue. There were no page or console
errors.

The privacy page explains browser/server storage and provides a local deletion
instruction plus a `mailto:` server-deletion request route. The demo set no
cookies and made only same-origin requests. Receipt bytes stayed in IndexedDB
and outside localStorage and server request bodies. Service-worker update and
offline `/demo` reload passed.

`/privacy`, `/terms`, `/records`, `/share/demo`, `robots.txt`, and `sitemap.xml`
returned their expected pages. The unknown route returned the designed page
with genuine HTTP 404; only its wording is a finding.

Fresh mobile Lighthouse on `/demo` scored 100 performance, 100 accessibility,
100 best practices, and 100 SEO. FCP was 1.20 s, LCP 1.29 s, TBT 40 ms, CLS 0,
and total transfer 73,257 bytes. Raw output is
`/work/.evidence/verification-22-lighthouse.json`.

## Backend, isolation, and deployment evidence

`EXPECTED_BUILD_SHA=7c840e... npm run verify:release` passed and reported:

- Single revision mode, min/max replicas 1/1, and one running replica.
- Product-owned Azure Files mounted at `/data`.
- Immutable `sf-mtd-quarterly-ready` image.
- Live build SHA `7c840e4853bbcb16270977bdb568271ebd86c746`.
- `not_configured` HMRC mode with no direct-submission capability.
- 20/20 concurrent acknowledged workspace documents retained.
- Forty reads allowed, then 429 with positive `Retry-After`.
- Twelve writes allowed, then 429 with positive `Retry-After`; the OAuth
  callback shared that write allowance.
- Both monthly and annual checkout endpoints returned allowed Dodo HTTPS URLs.

A separate local two-start exercise wrote ten isolated workspaces and an
encrypted accountant link, stopped the release binary, started a second runtime
directory against the same durable data directory, and restored every item.
The first start logged a generated key and the second a persisted key without
printing its value. The verification marker was absent from the durable files
as plaintext.

The isolation cleanup itself passed all focused checks:

- `bash -n scripts/deploy-container.sh` passed.
- `npm run test:deploy-contract` passed.
- `DEPLOYMENT_MODE=approved` exited 2 before any build or infrastructure action.
- Executable deployment configuration contained no shared-vault lookup, vault
  command, secret reference, or HMRC secret variable.
- The release command requires topology verification and the live
  `not_configured` state.

No live deployment, live restart, infrastructure mutation, secret lookup, or
direct SQLite-file mutation was performed. Live test records used isolated
random QA workspace IDs through the documented API; the restart exercise used
only temporary local directories.

## Runtime and source parity

The live container still identifies as `7c840e4`, which is expected because the
later isolation commit changes deployment/release scripts and documentation,
not the compiled frontend or backend. The current clean build and live copies
of `index.html`, JavaScript, CSS, and `sw.js` had identical SHA-256 hashes.

The implementation under this work order is `3e34f53`; `1c89363` only refreshes
Graphify output. No fresh product image is required for those non-runtime
changes.

## Earlier finding disposition

| Earlier finding | Current evidence |
| --- | --- |
| Broken clean-checkout claims, absolute test path, typecheck, or cold timeout | Fixed; 24/24 exact claims and the 54-test clean suite pass in a fresh clone. |
| Empty-records console 404 | Fixed; cold `/records` returns a successful empty document with no console error. |
| Dead checkout or weak checkout verification | Fixed; both live checkout endpoints return allowed hosted URLs and retry coverage passes. |
| Missing server licence gate | Fixed; unauthorised live-link creation returns 402 and safe positive QA coverage passes. |
| Malformed dates, amounts, categories, or quarter bounds accepted | Fixed; browser and server reject them atomically and recovery tests pass. |
| Quarter rollover/separation and unregistered persistence claims | Fixed; current/future quarters use distinct browser keys and server workspace IDs; exact claims pass. |
| Receipt quota failure or server receipt copy | Fixed; three 1.4 MB receipts and quota recovery pass; bytes remain browser-only. |
| Demo share loses sample identity | Fixed; the persistent demo banner and controls remain on `/share/demo`. |
| Small touch targets, route metadata, missing manifest, or HTTP-200 unknown route | Fixed except for the newly reported 404 wording; target, metadata, PWA, and HTTP status tests pass. |
| Safe QA fixture missing | Fixed; live non-charging/non-filing fixture is observable. |
| Ephemeral multi-replica state, lost concurrent saves, or multiplied limits | Fixed; one durable replica, 20/20 concurrent saves, restart restore, and exact external limits pass. |
| Unlisted receipt, quarter, and conditional-submission claims | Fixed; each now has a manifest entry and passing exact test. |
| Paid-tier navigation race | Fixed; exact claim and repeated full suites pass. |
| Copy implied a deployed HMRC sandbox | Fixed; live copy says no integration is configured and does not claim filing. |
| Approved live HMRC submission missing | **Open; critical finding above.** |

Pre-existing `graphify-out/` changes were preserved and were not staged.
