# Verify UK records-to-quarter handoff — verification 25

**Verified:** 5 September 2026 UTC
**Verdict:** **PASS — 0 findings, 0 untested claims**
**Current milestone:** M1 — records to reviewed handoff
**Implementation reviewed:** `575aa8659469ab6f45bce623c3da560a82248895`
**Documentation commit:** `bc81f80dbc0d2a8f88741f4f777fbb7f36faf604`
**Clean checkout / live build label:** `73ddca8e56958abbdccff19a140d505f6bd8527a`
**Live image:** `sociobotregistry.azurecr.io/sf-mtd-quarterly-ready@sha256:06a13cf9ca86518490bcc5ac4257b63d97fe4a7fa05b744f22753442746a316e`
**Live URL:** <https://mtd-quarterly-ready.sociobot.in>

## Decision

**PASS.** The plain-words repair closes Verification 24's only finding. A cold visitor can see the M1 job, audience, and first action on both required viewports; the one-click sample shows a useful, labelled quarter; all declared public claims have directly run evidence; and the deployed backend retains the single-replica, durable, handoff-only M1 safeguards.

The implementation candidate is `575aa86`. Live `/health` identifies the report/Graphify descendant `73ddca8`; its diff from `575aa86` contains only factory documentation, Graphify output, and a browser regression test. It contains no runtime Rust, frontend, public asset, dependency, Docker, or build configuration change. The live runtime is therefore the reviewed implementation, with its later documentation/build label recorded separately.

## Cold first screen and sample outcome

Fresh Chromium desktop (1440 × 900) and phone (390 × 844) contexts opened the live root without scrolling. Both had zero console or page errors and no horizontal overflow. The title was **Quarterly Ready — Check your MTD quarter** and both screens exposed:

- Job: **Turn records into a checked quarterly update**.
- Audience: **For UK sole traders, tutors and landlords who need MTD records without a full accounting suite.**
- First action: **Try it with sample data**.

The primary action was in the first viewport on desktop and phone. Evidence: `/work/.evidence/verification-25/live-cold-desktop.png` and `/work/.evidence/verification-25/live-cold-phone.png`.

The live browser regression then opened the sample. It passed the persistent **Demo — sample data, nothing is saved** label, realistic populated Maya Patel Tutoring quarter, quarter status, accountant CSV and sample-link controls, reset, and real-data isolation checks. The `@claim:demo-isolation` test also asserted the separate `demo:` namespace, no cookies, and only same-origin traffic. This is an isolated sample: changing or resetting it does not enter a real workspace.

## Claims and clean checkout

A new clone at `73ddca8` completed `npm ci` (60 packages, zero audit vulnerabilities). Every exact command in `.factory/claims.json` was run individually from that checkout. All **24/24** passed; command logs are in `/work/.evidence/verification-25/claims/`.

| Claim group | Result |
| --- | --- |
| Demo, privacy, sample access, CSV, review, import, receipts, quarter persistence and separation | Pass |
| Conditional consent/submission guards, HMRC handoff, accountant link and server subscription gate | Pass |
| Encryption, audit chain, anonymous page count, offline browser copy, paid tier, non-filing sandbox guard and API limits | Pass |

`npm test` passed **56/56**: type check, 11 Vitest tests, 18 Rust tests, deployment contract, production build, and browser suite. The production build created `dist/` with 15.32 kB gzip JavaScript and 5.32 kB gzip CSS.

## Live behaviour, accessibility, routes, and recovery

The live suite covered desktop and phone layout, keyboard demo entry, visible focus, dialog focus, 200% text, reduced motion, offline demo reload, privacy, legal pages, route titles/focus, internal links, and the genuine designed 404 with Return home recovery. Axe integration reported no serious or critical violations on `/`, `/demo`, `/privacy`, and `/terms`.

`npm run verify:url` passed live for `/demo`, `/privacy`, and `/terms`: each has its route title, `lang=en-GB`, exactly one main landmark and H1, no missing image alternatives, and zero console errors. `robots.txt`, `sitemap.xml`, and the deliberate HTTP 404 all responded correctly.

One unconfigured full-live-suite run correctly reported its expected build-identity mismatch (`dev` versus the live SHA); it was a verifier command omission, not a product failure. With the documented `EXPECTED_BUILD_SHA`, a Chromium process separately crashed before opening one test context and a fixed, previously-used verification IP hit the deliberate write allowance in the invalid-input loop. Neither was treated as a product finding: the targeted mobile target test then passed, and a fresh-client live probe received seven 422 validation responses for invalid records and confirmed `{"document":null}` afterward. All public claim tests in the live suite passed; its one skipped anonymous-page-count test is explicitly ingress-only, not a claim or a missing product path.

## Backend, privacy, durability, and limits

`EXPECTED_BUILD_SHA=73ddca8… npm run verify:release` passed against live. It proved:

- an immutable image digest; Single active-revision mode and exactly one min/max/running replica;
- the product Azure Files mount at `/data`;
- a saved workspace round trip and 20/20 concurrent acknowledged saves;
- malformed, impossible-quarter, out-of-quarter, zero-value, and unknown category records are rejected;
- permitted monthly and annual Sociobot checkout URLs plus the safe, non-charging/non-filing fixture;
- 40 read and 12 write requests admitted, then 429 with positive `Retry-After`, including the OAuth callback; and
- `hmrc_integration_configured:false` and `hmrc_integration_mode:"not_configured"`, which is the required honest handoff-only M1 state.

No advertising cookies or third-party analytics were observed in the demo claim. Demo traffic is same-origin; receipt content remains browser-local under the claim tests.

## Earlier finding disposition

| Earlier finding | Current disposition |
| --- | --- |
| Decorative/mood labels on landing, app, legal, and 404 surfaces (Verification 24) | **Closed.** Direct task/result labels, plain first screen, copy audit, and desktop/phone outcome regression pass live. |
| 404 metaphor/recovery wording (Verification 22) | **Closed.** Designed real 404, plain Page not found heading, title, and Return home path pass. |
| Lost saves, replicas greater than one, absent durable mount, mutable release proof, and concurrency loss | **Closed.** Release verification proves one replica, Azure Files `/data`, immutable digest, persistence, and 20 acknowledged concurrent documents. |
| Rate-limit absence or aggregate over-allowance | **Closed.** The live stable-client verifier proves 40/12 allowances and 429 plus positive `Retry-After`. |
| Empty/invalid records, receipt quota recovery, cross-quarter loss, demo isolation, receipt locality, offline reload, accessibility, metadata, or legal/404 route failures | **Closed.** Clean and live browser coverage passed the normal, invalid, boundary, recovery, mobile, keyboard, privacy, and offline paths. |
| Checkout and client-only live-link guard | **Closed for current M1 claims.** Both controller destinations are available and unlicensed live links are server-rejected. This is not evidence of the future paid-customer lifecycle. |
| Missing approved provider, taxpayer consent, and HMRC submission | **Not an M1 defect.** The live product truthfully hides direct submission and is handoff-only. It remains the separately blocked M3 external dependency, not a shipped capability. |

## Milestone and external dependencies

**M1 is accepted:** a UK sole trader, tutor, or landlord can capture/import quarter records, keep receipt evidence in the browser, review unresolved work, and download CSV/HMRC handoff material or use the isolated accountant sample. It does not file a tax return.

**M2 is not shipped:** authenticated accounts, tenant-scoped access, cross-device recovery, and the authorised paid purchase/restore lifecycle. Its external dependencies are Sociobot Entra CIAM and the authorised Sociobot billing entitlement flow.

**M3 is not shipped:** approved-provider consent and controlled submission. Its external dependencies are an approved MTD provider contract, operator runtime configuration, taxpayer consent, and a controlled provider/HMRC acknowledgement. Live health confirms none is configured; the product makes no contrary public promise.

## Verification commands

```sh
npm ci
# each exact command in .factory/claims.json
npm test
EXPECTED_BUILD_SHA=73ddca8e56958abbdccff19a140d505f6bd8527a npm run verify:release
npm run verify:url -- https://mtd-quarterly-ready.sociobot.in/demo
```
