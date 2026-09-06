# Quarterly Ready — strict review 2

**Reviewed:** 6 September 2026 UTC
**Verdict:** **PASS — 0 findings, 0 untested claims**
**Controller stage:** M2 building
**Reviewed scope:** M2 account foundation; M1 remains the accepted public workflow
**Live URL:** <https://mtd-quarterly-ready.sociobot.in>

## Review identity

- **Implementation candidate:** `08b2470eaff37e998e30323e98c2f87d8750baf6`
- **Live health build label:** `0dcc20f5fec98a476540ea9f684cb92879de6903`
- **Independent verification report commit:** `64c6f3e`
- **Documentation base reviewed:** `4d5b4f3cd40ccd95f96d73e747d1afccb8e8d905`
- **Live immutable image:** `sociobotregistry.azurecr.io/sf-mtd-quarterly-ready@sha256:0323c799c17a1bcba78a1ddf74dfcf1b7fcd1f61674ff12d530483f698c4eb2b`

The live label is a descendant of the implementation candidate. Its diff from
`08b2470e` contains only the M2 handoff and Graphify output. The later
documentation base adds verification/handoff evidence and more Graphify
output. No runtime frontend, Rust, asset, dependency, Docker, or build file
differs from the implementation candidate.

## Decision

**PASS.** No finding of any severity and no untested public claim remains.
This is a pass for the implemented M2 foundation, not a statement that the
whole M2 milestone is customer-live. The live service truthfully keeps sign-in
unavailable until an authorised Sociobot Entra CIAM registration exists. M1
records, demo, free downloads, and reviewed handoff remain available.

## First screen and sample

Fresh Chromium contexts at 1440 × 900 and 390 × 844 showed before scrolling:

- job: **Turn records into a checked quarterly update**;
- audience: **For UK sole traders, tutors and landlords who need MTD records without a full accounting suite.**;
- first action: **Try it with sample data**.

Both contexts had no horizontal overflow, console error, page error, cookie, or
third-party request. Keyboard activation worked on desktop. One click opened
the sample with ten Maya Patel Tutoring transactions, £260.00 income, £155.83
costs, £104.17 net, and the persistent **Demo — sample data, nothing is saved**
label. After resolving the sample bank transfer, **Reset demo** restored ten
rows and one unresolved category. A real-storage sentinel stayed unchanged;
only the `demo:` sample key was added. Screenshots and the recorded observations
are in `/work/.evidence/review-2/`.

The interface uses direct task and result labels, including **Quarter status**,
**How it works**, **Quarter checks**, **Review your quarter**, and **Downloads
and sharing**. No decorative or mood heading from Verification 24 remains.

## Clean checkout and public claims

A clean clone at documentation base `4d5b4f3` completed `npm ci` with 60
packages and no audit vulnerability. All 27 exact commands in
`.factory/claims.json` ran independently and passed. Per-command logs and the
machine-readable summary are under `/work/.evidence/review-2/claims/` and
`/work/.evidence/review-2/claims-summary.json`.

| Passing claim group | Claim IDs |
| --- | --- |
| Demo, privacy, records, and downloads | `demo-isolation`, `demo-access`, `privacy-no-tracking`, `accountant-csv`, `quarter-review`, `free-quarter-persistence`, `csv-import`, `receipt-capture`, `receipt-locality`, `quarter-record-separation`, `hmrc-handoff`, `offline-browser-copy` |
| Conditional integrations and paid controls | `hmrc-submission`, `hmrc-consent-no-records`, `conditional-submission`, `accountant-link`, `accountant-link-expiry`, `server-licence-gate`, `paid-tier`, `hmrc-sandbox-no-filing` |
| Storage, audit, accounts, and limits | `encrypted-storage`, `audit-log`, `anonymous-page-count`, `tenant-isolation`, `account-migration`, `account-export-delete`, `api-rate-limit` |

The configured-provider claims pass against explicit fixtures. They do not
prove or claim that an approved provider is configured live. Likewise, the
three account claims prove the prepared signed-session boundary; they do not
substitute for the future authorised CIAM journey.

`npm test` passed TypeScript, 11 Vitest tests, 22 Rust tests, the deployment
contract, production build, and 59 Playwright tests. Formatting, Clippy with
warnings denied, and `git diff --check` also passed. `dist/` contains 17.51 kB
gzip JavaScript and 5.57 kB gzip CSS.

## Live browser, accessibility, and performance

The full live Playwright run passed 58 tests with one intentional ingress-only
fallback skip; that fallback passed in the local 59-test run. It covered normal,
invalid, boundary, recovery, receipt quota, offline service-worker update and
reload, keyboard, focus, reduced motion, 200% text, responsive, legal, link,
route-title, privacy, and designed 404 paths.

The first live accessibility pass had one Chromium process crash before a
mobile target-size assertion executed. A fresh rerun passed that assertion,
and the later complete live suite passed it again. This was runner instability,
not a product failure, and no check was left untested.

`verify:url` passed on `/`, `/demo`, `/account`, `/privacy`, and `/terms` with
the correct title, `lang=en-GB`, one main landmark, one H1, complete image text,
and zero console errors. Playwright Axe found no serious or critical issue on
all five routes. Internal links passed; `robots.txt` and `sitemap.xml` return
200. The designed missing route correctly returns HTTP 404 and its **Return
home** path works.

Mobile Lighthouse measured performance 100, accessibility 100, best practices
100, and SEO 100. LCP was 1.437 s, CLS 0, and total blocking time 7 ms.

## Backend, account boundary, persistence, and limits

`EXPECTED_BUILD_SHA=0dcc20f5… npm run verify:release` passed against live. It
proved the immutable image, Single revision mode, one min/max/running replica,
the existing product Azure Files `/data` mount, durable save/read, 20/20
concurrent acknowledged documents, safe non-charging/non-filing fixtures, both
Sociobot checkout routes, and honest capability disclosure:
`accounts_configured:false`, `hmrc_integration_configured:false`, and
`hmrc_integration_mode:not_configured`.

A fresh verifier workspace was saved, read, and then read again after restarting
only active product revision `sf-mtd-quarterly-ready--0000085`. Its exact
encrypted record survived, and health returned the same live build. Existing
customer workspaces and deployment configuration were not changed.

Live rate-limit probes admitted 40 reads and 12 writes per client, then returned
429 with a positive `Retry-After`; the OAuth callback shared the 12-write
allowance. Health remains exempt.

Code and isolated-server tests show that the M2 routes derive the subject from
a signed, hashed, HTTP-only session, validate OIDC issuer/audience/signature,
use PKCE/state/nonce, verify business membership before account reads and
writes, return a non-enumerating not-found response across tenants, encrypt
account quarters, and preserve an idempotent explicit migration. Account export
and self-service deletion remove one account without touching another. Live
`/account` instead explains that sign-in is unavailable and does not alter
browser records.

## Earlier finding disposition

| Earlier issue | Current disposition |
| --- | --- |
| Verification 24 decorative and mood labels, including first-screen labels | **Closed.** Fresh desktop/phone checks and the copy audit use direct task/result names. |
| Verification 22 404 wording and status | **Closed.** Direct copy, genuine HTTP 404, and working recovery passed live. |
| Historical empty/invalid input, quarter separation, receipt quota/locality, demo isolation, offline, metadata, legal, keyboard, focus, target-size, and contrast issues | **Closed.** Independent claim commands plus the local and live suites passed. |
| Historical lost saves, concurrent-write loss, mutable image, replica topology, `/data`, and request-limit failures | **Closed.** Release, restart, concurrency, topology, and 429 evidence passed. |
| Checkout routing and client-only live-link gate | **Closed for current public claims.** Both real controller routes respond and the server rejects an unverified subscription. A purchased-customer lifecycle remains an M2 completion dependency. |
| Missing approved provider, taxpayer consent, and filing acknowledgement | **Not a current shipped claim.** Live capability remains explicitly unavailable; this is the separate M3 dependency. |

## Milestone and external dependencies

- **M1 accepted and public:** capture/import quarter records, retain receipt
  evidence in the browser, resolve checks, and download CSV or a reviewed
  handoff. It does not file a return.
- **M2 foundation reviewed and passing, milestone still building:** OIDC/PKCE,
  tenant-scoped storage, explicit migration, account export/delete, and
  account-scoped links are implemented. Authorised Sociobot Entra CIAM
  registration and an authorised billing purchase/restore/revoke lifecycle are
  still required before customer-live account, recovery, or completed-M2 claims.
- **M3 unshipped:** an approved MTD provider contract, runtime configuration,
  taxpayer consent, and a controlled provider/HMRC acknowledgement remain
  external dependencies. No public screen says they are available.

The missed-leverage review found no absent current-milestone feature: CSV
import/export, offline recovery, and accountant sharing are present. Sending
financial records to an AI model would not improve the conservative compliance
job and is correctly absent.
