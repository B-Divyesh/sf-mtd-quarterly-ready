# Quarterly Ready — verification 17 handoff

## Release status: FAIL

Candidate `4c20b33de43b97b1444a541314546159d67cc5d5` is deployed at
<https://mtd-quarterly-ready.sociobot.in>, but it must not be released.

- Production `/health` reports `hmrc_integration_configured: false`; the
  approved HMRC direct-submission capability required by the brief is absent.
- The deployed service acknowledges real workspace saves with HTTP 200 but
  loses records: the live persistence claim failed three consecutive times and
  only 1 of 10 concurrent save/read probes persisted.

See [verification-17.md](verification-17.md) for exact commands and evidence.

## What was verified

- Clean `npm ci`, every exact command in `.factory/claims.json`, `npm test`,
  formatting, clippy, and a release Rust build all passed locally.
- The live build identity matches the candidate SHA. The page has a clear
  one-click demo, no observed console errors or third-party demo requests,
  working response security headers, a passing 390 px/keyboard/reduced-motion
  accessibility suite, and passing offline service-worker behavior.
- The live rate policy allows 40 reads or 12 writes per persistent client,
  then returns 429 with `Retry-After: 1`.
- `npm run verify:release` fails as intended because the approved HMRC
  integration is not configured.

## Required next steps

1. Correct the deployed persistence boundary (replicas, durable volume,
   encryption key, and restore behavior), then prove concurrent writes and
   post-restart reads for all workspaces.
2. Provision and test a genuine approved HMRC MTD provider and taxpayer-consent
   flow. Do not represent the existing handoff-only mode as direct submission.
3. Redeploy and rerun the full claims gate, live persistence repetitions,
   `EXPECTED_BUILD_SHA=<new-sha> npm run verify:live`, and
   `EXPECTED_BUILD_SHA=<new-sha> npm run verify:release`.

Docker is unavailable in this verifier container, so an image build was not
repeated. No product code changed. Pre-existing `graphify-out/` modifications
were preserved and excluded from this verification.
