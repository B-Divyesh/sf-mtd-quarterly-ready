# Quarterly Ready — independent verification 26

**Verified:** 6 September 2026 UTC
**Verdict:** **PASS — 0 findings, 0 untested claims**
**Current controller stage:** M2 building
**Current public milestone:** M1 records-to-reviewed-handoff remains accepted; M2 is an implemented account foundation, not a customer-live account release.
**Implementation reviewed:** `08b2470eaff37e998e30323e98c2f87d8750baf6`
**Documentation commit:** `ef9442547aae1c0caff1a732b78cf4b89667a980`
**Live build label:** `0dcc20f5fec98a476540ea9f684cb92879de6903`
**Live URL:** <https://mtd-quarterly-ready.sociobot.in>

## Decision

**PASS.** The implementation candidate is live. The live build label is a later descendant; its diff from the implementation contains only handoff/Graphify output, with no runtime frontend, Rust, public asset, dependency, Docker, or build-configuration change. It is therefore the reviewed implementation at runtime.

All 27 declared claim commands passed independently from a clean checkout. The complete suite, live accessibility checks, URL checks, sample/reset flow, release/durability probe, and request-limit probe passed. No finding of any severity and no untested public claim remains.

## First screen and one-click sample

Fresh Chromium contexts at 1440 × 900 and 390 × 844 showed, before scrolling:

- Job: **Turn records into a checked quarterly update**.
- Audience: **For UK sole traders, tutors and landlords who need MTD records without a full accounting suite.**
- First action: **Try it with sample data**.

The live phone sample opened with ten Maya Patel Tutoring transactions, £260.00 income, £155.83 costs, £104.17 net, and the persistent **Demo — sample data, nothing is saved** label. After categorising the sample bank transfer, **Reset demo** restored ten rows and one unresolved category; browser errors were zero. The live `@claim:demo-isolation`, `@claim:demo-access`, and `@claim:quarter-review` run separately proved demo storage, no cookies or third-party requests, and the category-review outcome.

## Clean checkout, claims, and build

A clean clone at `0dcc20f5fec98a476540ea9f684cb92879de6903` completed `npm ci` with 60 packages and zero vulnerabilities. Every exact command in `.factory/claims.json` passed independently; command logs are in `/work/.evidence/verification-26/claims/`.

| Claim IDs with passing exact-command evidence | Result |
| --- | --- |
| `demo-isolation`, `demo-access`, `privacy-no-tracking`, `accountant-csv`, `quarter-review`, `free-quarter-persistence`, `csv-import`, `receipt-capture`, `receipt-locality`, `quarter-record-separation` | Pass |
| `hmrc-submission`, `hmrc-consent-no-records`, `conditional-submission`, `hmrc-handoff`, `accountant-link`, `accountant-link-expiry`, `server-licence-gate` | Pass |
| `encrypted-storage`, `audit-log`, `anonymous-page-count`, `offline-browser-copy`, `paid-tier`, `hmrc-sandbox-no-filing`, `tenant-isolation`, `account-migration`, `account-export-delete`, `api-rate-limit` | Pass |

`npm test` passed: TypeScript, 11 Vitest tests, 22 Rust tests, deployment contract, production build, and 59 Playwright tests. `test-results/.last-run.json` records `status: passed`. The build produced `dist/` with 17.51 kB gzip JavaScript and 5.57 kB gzip CSS.

## Live paths, accessibility, privacy, and routes

The targeted live browser run passed the fresh desktop/phone plain-words regression, account-unavailable recovery, 390 px no-overflow check, keyboard demo entry, and designed HTTP 404 recovery. Live Axe scans had no serious or critical violations on `/`, `/demo`, `/account`, `/privacy`, and `/terms`.

`npm run verify:url` passed for those five routes: each has its required route title, `lang=en-GB`, one main landmark, one H1, no missing image alternatives, and zero console errors. `robots.txt` and `sitemap.xml` return 200. The intentional unknown route returns 404 and its recovery link works.

The live demo isolation claim observed no cookies, no third-party analytics, only same-origin requests, and no account-session or business requests. Local claim evidence also covers service-worker offline reload, receipt locality, invalid import recovery, quarter separation, keyboard/focus, reduced motion, 200% text, downloads, legal links, and privacy boundaries.

## Backend, persistence, and rate limits

`EXPECTED_BUILD_SHA=0dcc20f5fec98a476540ea9f684cb92879de6903 npm run verify:release` passed against live. It verified:

- immutable image `sociobotregistry.azurecr.io/sf-mtd-quarterly-ready@sha256:0323c799c17a1bcba78a1ddf74dfcf1b7fcd1f61674ff12d530483f698c4eb2b`;
- Single revision mode, exactly one min/max/running replica, and the product Azure Files `/data` mount;
- durable workspace read-after-write and 20/20 concurrent acknowledged documents retained;
- invalid transaction and quarter input rejection;
- permitted monthly and annual Sociobot checkout URLs plus the explicitly non-charging, non-filing QA fixture;
- 40 reads and 12 writes admitted per client, then 429 with a positive `Retry-After`, including the OAuth callback; and
- honest handoff-only health: `hmrc_integration_configured:false`, `hmrc_integration_mode:"not_configured"`, and `accounts_configured:false`.

The new account-boundary claims (`tenant-isolation`, explicit idempotent migration, and account export/delete) passed as isolated server tests. The live `/account` page truthfully says account sign-in is unavailable and leaves browser records unchanged; it does not falsely present M2 as customer-live.

## Earlier finding disposition

| Earlier finding | Current disposition |
| --- | --- |
| Verification 24 decorative/mood labels, including the first desktop and phone screens | **Closed.** The fresh desktop/phone regression finds direct task labels, `Quarter status`, and `Downloads and sharing`; the copy audit inventories the public labels. |
| Verification 22 404 wording/status | **Closed.** The designed page has direct `Page not found` copy, true HTTP 404, and Return home recovery. |
| Historical durability, replica, mutable-image, concurrency, and rate-limit defects | **Closed.** The live release verifier proved an immutable image, one replica, `/data`, 20 retained concurrent saves, and 40/12 allowance followed by 429/`Retry-After`. |
| Historical empty/invalid data, demo isolation, receipt, offline, accessibility, metadata, legal, and route defects | **Closed.** Clean claim commands, 59 local browser tests, live Axe/URL checks, and the live demo/reset path pass. |
| Checkout availability and server-side live-link gate | **Closed for the current public promises.** Live release verification found both controller checkout URLs and the declared gate tests pass. A purchased customer lifecycle is not claimed yet. |
| Missing approved provider/consent/submission | **Not a current defect.** It is the separately unshipped M3 dependency. Live health and copy remain honestly handoff-only. |

## Milestone and external dependencies

**M1 accepted and public:** UK sole traders, tutors, and landlords can capture/import quarter records, keep receipt evidence in the browser, resolve checks, and download CSV or a reviewed HMRC handoff. It does not file a tax return.

**M2 building, not customer-live:** the repository contains the prepared OIDC/PKCE, tenant-scoped records, migration, export/delete, and account-link foundation. An authorised Sociobot Entra CIAM issuer/client registration and authorised Sociobot billing entitlement/restore lifecycle are external dependencies before real sign-in, tenant recovery, purchase, restore, revoke, or paid-customer claims can be accepted.

**M3 unshipped external dependency:** an approved MTD provider contract, product runtime configuration, taxpayer consent, and a controlled provider/HMRC acknowledgement. None is configured live, and no public copy claims otherwise.

## Verification commands

```sh
npm ci
# Run every exact command in .factory/claims.json independently
npm test
EXPECTED_BUILD_SHA=0dcc20f5fec98a476540ea9f684cb92879de6903 npm run verify:release
npm run verify:url -- https://mtd-quarterly-ready.sociobot.in/demo
```
