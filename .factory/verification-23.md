# Verify UK records-to-quarter handoff — verification 23

**Verified:** 5 September 2026 UTC

**Verdict:** **FAIL — 1 finding, 0 untested claims**

**Current milestone:** M1 — records to reviewed handoff

**Implementation reviewed:** `13380e4b15634ce808be5198f126eea1ce088d82`

**Documentation/handoff commit:** `02d94b52ce4be9a3a85bf65a2d74e360b45fb442`

**Clean checkout and live build identity:** `89338a9a477c6033b553fdb0e658a23e614712c8`

**Live URL:** <https://mtd-quarterly-ready.sociobot.in>

## Decision

The M1 application behavior passes. All 24 declared claim commands pass from a
fresh checkout, `npm test` passes, the complete browser regression covers the
repaired 404 copy, and the live records-to-handoff workflow works on desktop
and phone. Approved HMRC submission remains unavailable and honestly hidden;
under the accepted venture plan this is an M3 external dependency, not an M1
finding.

The current release nevertheless fails one mandatory deployment check. The
live Container App revision references the image by mutable tag rather than an
immutable digest. `npm run verify:release` fails on that exact condition.
Therefore this verification is a product **FAIL**, despite the passing product
behavior and zero untested claims.

## Finding

### High — the live revision uses a mutable image tag

Read-only inspection of `sf-mtd-quarterly-ready` returned:

```json
{
  "activeRevisionsMode": "Single",
  "minReplicas": 1,
  "maxReplicas": 1,
  "image": "sociobotregistry.azurecr.io/sf-mtd-quarterly-ready:89338a9a477c",
  "latestRevision": "sf-mtd-quarterly-ready--0000077"
}
```

The `/data` Azure Files mount is present and exactly one replica is running.
However, the repository's required release command failed with:

```text
topology check failed: container image must use an immutable digest, got
sociobotregistry.azurecr.io/sf-mtd-quarterly-ready:89338a9a477c
```

A mutable tag does not pin the running revision to one content-addressed image
and contradicts the deployment contract and the previous handoff claim that
the live release verifier passes.

Resolution is operational: redeploy the already-tested application using its
`@sha256:` image reference, without changing product code, then rerun
`EXPECTED_BUILD_SHA=<live-sha> npm run verify:release`.

## Milestone and external dependencies

M1 is the accepted non-filing workflow: capture/import records, attach local
receipts, review a quarter, download accountant CSV and HMRC-ready handoff
files, and open the sample accountant pack. No M1 feature depends on an
external HMRC provider.

The live health response is explicit:

```json
{
  "hmrc_integration_configured": false,
  "hmrc_integration_mode": "not_configured",
  "hmrc_taxpayer_consent_required": false,
  "hmrc_provider_name": null
}
```

M2 accounts, authenticated tenant isolation, and a proved paid-customer
lifecycle are not shipped. Browser-held workspace IDs separate current M1
records, but are not an identity boundary. M3 approved-provider submission and
taxpayer consent require a separately authorised provider contract and runtime
configuration. The mock submission and non-filing QA fixture do not count as a
live filing capability.

## First screen and fresh-browser result

Fresh 1440 × 900 desktop and 390 × 844 phone contexts were opened at the live
landing page without scrolling. Both showed:

- Job: `Turn records into a checked quarterly update`.
- Audience: UK sole traders, tutors, and landlords who need MTD records without
  a full accounting suite.
- First action: `Try it with sample data`, beside `Opens a private sample
  quarter. No account needed.`

All three elements were inside the initial viewport. The page title was
`Quarterly Ready — Check your MTD quarter`. Phone document and viewport width
were both 390 px. Neither context produced console/page errors, cookies, or a
cross-origin automatic request.

Screenshots are at:

- `/work/.evidence/verification-23/live-cold-desktop.png`
- `/work/.evidence/verification-23/live-cold-phone.png`
- `/work/.evidence/verification-23/live-demo-phone.png`
- `/work/.evidence/verification-23/live-demo-reset-desktop.png`

## One-click sample, output, reset, and isolation

The first action opened `/demo` in one click. The persistent label said
`Demo — sample data, nothing is saved` and remained present on the read-only
sample accountant pack. The sample showed Maya Patel Tutoring, ten realistic
transactions, £260.00 income, £155.83 costs, £104.17 net, one uncategorised
transaction, and one missing receipt.

The unresolved bank transfer was assigned to Sales and the figures were
confirmed. The accountant CSV downloaded with 19 CRLF lines and included the
business and sample transfer. The reviewed handoff contained:

```json
{
  "format": "quarterly-ready-mtd-itsa-handoff-v1",
  "periodStartDate": "2026-04-06",
  "periodEndDate": "2026-07-05",
  "turnover": 260,
  "reviewedByUser": true
}
```

`Reset demo` restored all ten rows and the unresolved category. A separate
real-storage sentinel was unchanged. The complete sample flow made zero
`/api/workspace` requests, used only the product origin, set no cookies, and
reported no browser errors.

## Claims and clean-checkout gates

A fresh clone at `89338a9` was used. After the documented `npm ci`
prerequisite, every exact `test` command in `.factory/claims.json` was run
individually. Result: **24 passed, 0 failed, 0 untested**.

The 18 browser claims passed for demo access/isolation, privacy, CSV and
handoff output, quarter review/persistence/separation, CSV import, receipt
capture/locality, conditional consent/submission behavior, accountant links,
the server licence gate, offline reload, paid checkout wiring, and the exact
40-read/12-write request allowances. The six Rust commands passed for approved
provider payload logic, link expiry, encrypted storage, audit chaining, daily
anonymous page counts, and the non-filing sandbox boundary.

The landing page, application, privacy page, terms, README, demo guide, and
claim manifest were cross-checked. No material current M1 capability or privacy
claim lacks an observable test. Conditional provider tests prove only guarded
fixture behavior, which the copy and health response distinguish from live
availability.

| Clean-checkout check | Result |
| --- | --- |
| `npm ci` | Pass; 60 packages, 0 vulnerabilities |
| Every exact command in `.factory/claims.json` | Pass; 24/24 |
| `npm test` | Pass; typecheck, 11 Vitest, 18 Rust, deploy contract, build, 55 Playwright |
| `cargo fmt --all -- --check` | Pass |
| `cargo clippy --all-targets --all-features -- -D warnings` | Pass |
| `npm audit --audit-level=high` | Pass; 0 vulnerabilities |
| release Rust build with implementation SHA | Pass |
| `dist/index.html` produced | Pass |

The production bundle is 48.36 kB JavaScript (15.59 kB gzip) and 21.71 kB CSS
(5.33 kB gzip). The mobile hero is 23.00 kB.

## Live normal, invalid, boundary, and recovery paths

The first complete live browser run produced 53 passes, one expected ingress
skip, and one identity-only failure because the supplied expected SHA was
`13380e4` while `/health` now reports `89338a9`. Every functional test in that
run passed. The live frontend, JavaScript, CSS, and service-worker hashes match
the clean production build, and `git diff` confirms no runtime source changed
between `13380e4` and `89338a9`.

A second immediate run with the actual live SHA passed identity and all other
paths except one fixed-IP invalid-input test that met the intentionally shared
write quota left by the first run. An equivalent fresh-client probe then
returned 422 for all seven malformed records and 200 with `document:null` on
read, proving atomic rejection. This is expected limiter state from back-to-
back verification, not a product defect.

Passing live coverage includes:

- normal capture, current/next-quarter separation, CSV import, receipt
  attachment, category review, downloads, and read-only sample sharing;
- impossible dates, out-of-quarter dates, zero and over-limit amounts,
  unknown categories, receipt size/quota recovery, malformed server records,
  unreviewed submission, unauthorised live sharing, and checkout retry;
- a deliberate HTTP 404 with title `Page not found — Quarterly Ready`, H1
  `Page not found`, direct explanation, and `Return home` recovery in both the
  server and in-app render paths;
- route titles/focus, in-app route recovery, every same-origin link,
  `robots.txt`, `sitemap.xml`, `/privacy`, `/terms`, and the privacy/support
  email links.

## Accessibility, privacy, offline behavior, and performance

The live suite's Axe integration found no serious or critical violations on
`/`, `/demo`, `/privacy`, and `/terms`. Independent `verify:url` passed title,
`lang="en-GB"`, one H1, one main landmark, image alternatives, and console
checks. Keyboard demo entry, dialog focus and operation, visible focus, 44 px
mobile targets, 200% text, heading order, 390 px layout, and reduced-motion
behavior passed.

The service worker updated and a fresh demo context reloaded offline with the
sample and `Offline — browser copy active`. Receipt bytes stayed in IndexedDB,
outside localStorage and server request bodies. Automatic demo traffic stayed
same-origin with no cookies or third-party analytics. The privacy page explains
local deletion and provides an email route for server deletion requests.

Fresh mobile Lighthouse on `/demo` scored 100 performance, 100 accessibility,
100 best practices, and 100 SEO. FCP was 1.23 s, LCP 1.32 s, TBT 41 ms, CLS 0,
and total transfer 73,237 bytes.

## Backend and persistence

- `/health` returns 200, build `89338a9`, safe QA fixtures enabled, and the
  handoff-only HMRC capability.
- `npm run verify:live` passed: 20/20 concurrent acknowledged saves remained
  readable; both checkout endpoints returned allowed Dodo HTTPS URLs; the QA
  fixture remained explicitly non-charging/non-filing.
- The stable live probe admitted 40 reads then returned 429 on request 41, and
  admitted 12 writes then returned 429 on request 13. Both 429 responses had a
  positive `Retry-After`; the OAuth callback shared the write allowance.
- The release executable started with only `PORT`, generated its own key,
  reported the implementation SHA, and stopped cleanly.
- A separate two-start local exercise used a new live-database directory on the
  second start and the same durable directory. It reported a persisted key and
  restored 10/10 workspaces plus an encrypted accountant link. The plaintext
  marker was absent from the durable key and SQLite snapshot.
- Live topology is otherwise correct: one active revision, min/max replicas
  1/1, one running replica, and the product Azure Files mount at `/data`. The
  mutable image reference remains the finding.

## Earlier finding disposition

| Earlier finding(s) | Current disposition |
| --- | --- |
| Missing approved HMRC submission (verifications 1–22) | Not shipped and not claimed as M1. The accepted plan makes it a separate M3 provider/consent dependency; live health and UI remain explicitly handoff-only. |
| Clean-checkout claim harness, absolute test path, typecheck, Rust image tag, and startup log | Closed. Fresh clone claims and `npm test` pass; Dockerfile uses `rust:1-alpine`; port-only startup emits generated/persisted configuration without a secret value. |
| Client-only accountant-link gate, wrong one-time pricing, weak checkout test, dead checkout endpoints | Closed for current claims. Server rejects an unlicensed link with 402; monthly/annual copy and controller endpoints pass. Purchase/restore remains future M2 scope and is not claimed as complete. |
| Empty-workspace console 404 | Closed. Cold `/records` returns 200 with `document:null` and no console error. |
| Candidate/live code mismatch | Closed for application parity. Live build now identifies as the later report/Graphify SHA `89338a9`; runtime source and built asset hashes match `13380e4`. The mutable deployment reference is the new finding above. |
| Missing or multiplied external rate limits; unstable transport check | Closed. Stable keep-alive verification repeatedly proves 40 reads, 12 writes, 429, and positive `Retry-After` on one replica. |
| Small mobile targets, fixed route metadata, missing manifest, HTTP-200 unknown route | Closed. Target, metadata, PWA, and deliberate HTTP 404 tests pass. |
| Expired-quarter lock, malformed input, missing quarter separation/persistence claim | Closed. Current/future quarter and browser/server separation tests pass; invalid input is rejected atomically. |
| Demo accountant page lost its label | Closed. `/share/demo` keeps the banner, Reset demo, and Start for real. |
| Receipt quota crash or server receipt copy | Closed. Three 1.4 MB files, quota recovery, IndexedDB storage, and request-body locality pass. |
| Safe QA fixture absent or misleading HMRC sandbox copy | Closed. The observable fixture is enabled and explicitly says non-charging/non-filing; public copy says the live integration is not configured. |
| Ephemeral/multi-replica storage, lost concurrent saves, and missing topology check | Closed except for the new immutable-image finding. One replica and `/data` are present; concurrent and two-start restoration pass. |
| Unregistered receipt, quarter, privacy, and conditional-submission claims | Closed. The 24-entry manifest covers the current public claims and every command passes. |
| Paid-navigation race and missing repeatable accessibility helper | Closed. The retry regression and `verify:url` pass. |
| Minor metaphorical 404 wording in verification 22 | Closed. Both render paths use `PAGE NOT FOUND` / `Page not found`, direct recovery text, and a tested Return home action. |

## Evidence and scope

Detailed logs, JSON results, Lighthouse output, headers, hashes, and screenshots
are under `/work/.evidence/verification-23/`. No product code, live user data,
deployment, secret, or infrastructure was changed. Live backend probes used
only new random QA workspace IDs. The pre-existing unstaged `graphify-out/`
changes were left untouched.
