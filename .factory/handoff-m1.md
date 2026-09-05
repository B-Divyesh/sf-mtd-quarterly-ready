# Quarterly Ready M1 — accepted milestone handoff

## Milestone result

**M1 — records to reviewed handoff is accepted.** It covers the landing page, one-click isolated demo, transaction and receipt capture, CSV import, review checks, free accountant CSV, reviewed handoff JSON, and demo accountant pack. The product is intentionally non-filing.

The M1.1 repair deployed implementation `13380e4b15634ce808be5198f126eea1ce088d82`. It changed only the two 404 copy strings and their verification:

- real HTTP 404: `PAGE NOT FOUND` and `Page not found`;
- in-app fallback: the same direct wording;
- browser regression: genuine 404 status, clear recovery heading, Return home action, and client-side fallback recovery.

## Evidence

- Documented clean setup: `npm ci`, then `npm test` — passed (11 Vitest, 18 Rust, deployment contract, build, 55 Playwright).
- Local focused 404 route and browser recovery checks passed.
- Release verification against the deployed implementation passed with one immutable image, one running replica, the product Azure Files `/data` mount, 20/20 acknowledged concurrent saves, restart/revision durability, and 40-read/12-write external rate limits with `Retry-After`.
- `verify:url` passed live `/demo`; the live browser suite passed all 55 tests, including Axe integration.
- Fresh desktop and 390 px mobile contexts showed the required job, audience, and first action before scrolling. A fresh demo used realistic Maya Patel Tutoring data, kept its sample banner through changes, reset correctly, and did not change a real-data sentinel or call the workspace API.

## Boundaries and dependencies

M1 does not provide accounts, tenant isolation, a proven paid-customer lifecycle, taxpayer consent, or HMRC filing. The live health capability is `not_configured` for HMRC integration. Approved submission remains an M3 external dependency: a product-owned approved-provider contract and consent configuration, followed by an authorised controlled acknowledgement test. No credential or provider setting was added or read for this milestone.

## Independent verification entry points

- Live demo: `https://mtd-quarterly-ready.sociobot.in/demo`
- Claims: every exact `test` command in `.factory/claims.json`
- Local suite: `npm test`
- Live release check: `EXPECTED_BUILD_SHA=13380e4b15634ce808be5198f126eea1ce088d82 npm run verify:release`
