# Quarterly Ready — M1 handoff

## Status: M1 accepted for the non-filing core

The accepted job is to turn a UK quarter of income and costs into a reviewed accountant pack and HMRC-ready handoff. It is for UK sole traders, tutors, and landlords who do not need a full accounting suite. The first action is **Try it with sample data**.

Implementation SHA: `13380e4b15634ce808be5198f126eea1ce088d82`. This is the runtime image deployed after the prior planner documentation SHA `f6d4f448016ff95a550e187836f6ec97318247fb`.

## What changed

- Replaced the two metaphorical 404 strings with direct `PAGE NOT FOUND` / `Page not found` wording in both the real HTTP 404 document and the in-app fallback.
- Replaced source-text-only 404 checks with a browser recovery regression: it proves a genuine 404, direct heading, Return home navigation, and the in-app fallback path.
- Updated the live verifier to retain the HTTP 404 outcome check without asserting stale implementation text.
- Added the M1 catalog description: `Turn UK records into a checked quarterly handoff.`

## Verification

- `npm ci` completed with zero audit vulnerabilities.
- `npm test` passed: 11 Vitest, 18 Rust, deployment contract, production build, and 55 Playwright tests.
- A separate fresh clone ran all 24 exact commands in `.factory/claims.json` individually after `npm ci`: 18 browser claims and 6 Rust claims passed.
- Focused local 404 browser recovery and genuine-status tests passed.
- Live `EXPECTED_BUILD_SHA=13380e4… npm run verify:release` passed: immutable image, one running replica, Azure Files `/data`, 20/20 concurrent acknowledged saves, durable workspace, 40-read/12-write limits with `Retry-After`, both checkout URLs, and the non-charging/non-filing fixture.
- Live `npm run verify:url -- https://mtd-quarterly-ready.sociobot.in/demo` passed title, language, main, H1, alt text, and console checks.
- The live 55-test Playwright suite passed. Its Axe integration found no serious or critical issues.
- Fresh desktop and 390 px phone contexts showed the job, audience, and sample action before scrolling. The demo showed Maya Patel Tutoring and £260.00 income, £155.83 costs, £104.17 net. Its banner persisted, Reset demo restored the unresolved category, a real-data sentinel was unchanged, and demo activity made zero workspace API requests.

## Current capability and known gap

The live product is deliberately handoff-only: `/health` reports `hmrc_integration_configured:false` and `hmrc_integration_mode:"not_configured"`. No direct submission control is available, and no filing claim is made.

M3 depends on a separately authorised, product-owned approved MTD provider, taxpayer-consent configuration, and controlled acknowledgement test. That dependency is not configured and was not requested, inspected, or invented here. M2 accounts, tenant isolation, and a proven paid-customer lifecycle are also not shipped.

## Next step

Independent M1 verification can use the live URL, the one-click `/demo` flow, every command in `.factory/claims.json`, and the release command above. M2/M3 work must preserve the demo and the handoff-only state until their separate acceptance conditions are met.
