# Quarterly Ready — verification 18 handoff

## Status: FAIL — do not release

Independent verification of
`ca7cfc5e7f34547be6c9cd1963952952c2d28b82` at
<https://mtd-quarterly-ready.sociobot.in> is a **FAIL**. See
[verification-18.md](verification-18.md) for the complete evidence.

## Blocking defects

1. Production `/health` reports no approved HMRC integration
   (`hmrc_integration_configured: false`, `not_configured`). The brief requires
   reviewed submission through an approved integration.
2. The deployed API does not reliably enforce its documented rate limit. Fresh
   single-client probes receive normal validation/status responses at read 41
   and write 13, rather than `429` with `Retry-After`.

The previously reported concurrent real-workspace persistence loss was retested
and is fixed in this candidate: 10 simultaneous live save/read probes restored
10/10 correct records.

## Verification summary

- All 23 exact claims-manifest commands passed locally. Fresh live claims: 15/15
  passed; fresh live accessibility/Axe suite: 20/20 passed.
- Local typecheck, 11 Vitest tests, 18 Rust tests, deploy-contract check,
  formatting, Clippy, and release build passed. Two full `npm test` attempts
  were interrupted by supplied Chromium process crashes; isolated retries of
  the disrupted tests passed.
- Cold-page first-read, one-click demo, mobile/keyboard/reduced motion, privacy
  request log, headers, caching, bundle budgets, and candidate asset hashes
  passed. No Docker binary exists in this verifier container.

## How to run and verify

```bash
npm ci
npm test
EXPECTED_BUILD_SHA=ca7cfc5e7f34547be6c9cd1963952952c2d28b82 npm run verify:live
npm run verify:url -- https://mtd-quarterly-ready.sociobot.in
```

## Required external next step

An authorized operator must provision a real approved HMRC MTD provider and
its actual taxpayer-consent registration in Key Vault: submission URL, service
token, authorization URL, token URL, registered client id/secret, provider
name, and approval reference. Then deploy in approved mode, complete a real
taxpayer OAuth consent journey, submit a permitted test return through that
provider, and rerun `npm run verify:release`. Separately repair the deployed
rate limiter so every endpoint returns 429 with `Retry-After` after the
documented per-client allowance, and repeat the live stable-connection probe.
Do not change the release status to ready before both succeed.

Pre-existing `graphify-out/` changes were preserved and excluded from repair
commits.
