# Quarterly Ready — repair 17 handoff

## Status: handoff-only deployment healthy; direct HMRC release blocked

The repaired records, CSV, accountant-link, demo, and safe-fixture product is
live at <https://mtd-quarterly-ready.sociobot.in>. Its immutable deployed
application identity is source commit
`77a891d7f9f5a3b9f06882c369633b8852e03056` and image
`sociobotregistry.azurecr.io/sf-mtd-quarterly-ready@sha256:d8ad7fa549b553a65de524d37df2c29b859b62284dc4b8e667ad9a665fe7a881`.

This is deliberately a **handoff-only** deployment. A direct HMRC MTD release
is still blocked: the available Key Vault has no genuine approved-provider
submission, OAuth, client, or approval-reference secret set. No provider was
invented, no taxpayer consent was fabricated, and the UI does not claim or
offer direct filing.

## Repairs made

- Reproduced the verifier's concurrent-save loss case and added a ten-workspace
  concurrent save/read regression (`@regression:concurrent-workspace-saves-are-readable-before-the-success-response-is-trusted`).
- Reworked storage for Azure Files: SQLite runs on local container storage,
  every mutation is serialized, committed, copied to the mounted `/data`
  snapshot, and `sync_all` completes before HTTP success. Startup restores that
  snapshot. This avoids SQLite byte-range locking on Azure Files while keeping
  a durable one-replica hand-off.
- Added Rust restart coverage for encrypted workspaces, accountant links,
  audit rows, page counts, key continuity, and snapshot restore. Added the
  deployment probe that writes ten workspaces concurrently, restarts the
  replica, replaces the revision, and re-reads all records and the encrypted
  accountant link.
- Added a fail-closed approved-provider configuration, OAuth authorization-code
  taxpayer-consent endpoints, one-time state, encrypted consent-token storage,
  and consent-aware submission gating. The release script requires the full
  approved-provider Key Vault reference set before it will build or alter a
  release deployment.
- Pinned the live container to an ACR digest, set `Single` active revisions,
  exactly one replica, and an Azure Files mount at `/data`.

## Verification evidence

- Clean dependency install: `npm ci` (also performed by ACR build `ch18n`).
- Complete local gate: `npm test` passed on 2026-08-29: TypeScript, 11 Vitest,
  18 Rust tests, deploy contract, production Vite build (15.44 KB gzip JS;
  5.33 KB gzip CSS), and 51 Playwright tests. `cargo clippy --all-targets --
  -D warnings` and formatting passed.
- Live identity and product check:
  `EXPECTED_BUILD_SHA=77a891d7f9f5a3b9f06882c369633b8852e03056 npm run verify:live`
  passed. It verified the build SHA, safe non-charging/non-filing fixture,
  checkout URLs, 404, workspace save/read, validation, 40-read/12-write
  429+`Retry-After` policy, and response behavior.
- Live accessibility/browser check:
  `VERIFY_ORIGIN=https://mtd-quarterly-ready.sociobot.in npx playwright test tests/accessibility.spec.ts`
  passed 20/20, covering desktop, 390 px mobile, 200% text, keyboard demo
  path, focusable dialog, Axe serious/critical findings, reduced motion, links,
  route metadata, and no-console-error load. `npm run verify:url --
  https://mtd-quarterly-ready.sociobot.in` passed title/lang/main/alt/console.
- Deployment `ch18n` passed its durable proof: ten concurrent workspace saves
  plus an encrypted accountant link survived both a real replica restart and
  a revision replacement. The topology verifier reported `Single`, min/max 1,
  one running replica, the expected Azure Files share, and the immutable image
  digest above.
- The required direct-release gate was intentionally exercised and failed:
  `EXPECTED_BUILD_SHA=77a891d7f9f5a3b9f06882c369633b8852e03056 npm run verify:release`
  stops with `production has no approved HMRC integration configured`.
  `DEPLOYMENT_MODE=approved bash scripts/deploy-container.sh` also fails before
  building when the first required approved-provider Key Vault secret is absent.
- Lighthouse could not complete in this worker because the supplied Chromium
  process crashed while Lighthouse opened its tab. Playwright browser checks
  above passed; this runner limitation is the only missing measurement.

## How to run and verify

```bash
npm ci
npm test
EXPECTED_BUILD_SHA=77a891d7f9f5a3b9f06882c369633b8852e03056 npm run verify:live
npm run verify:url -- https://mtd-quarterly-ready.sociobot.in
```

`DEPLOYMENT_MODE=handoff-only bash scripts/deploy-container.sh` deploys the
honest non-filing product. `DEPLOYMENT_MODE=approved` is intentionally refused
until the provider configuration below exists.

## Required external next step

An authorized operator must provision a real approved HMRC MTD provider and
its actual taxpayer-consent registration in Key Vault: submission URL, service
token, authorization URL, token URL, registered client id/secret, provider
name, and approval reference. Then deploy in approved mode, complete a real
taxpayer OAuth consent journey, submit a permitted test return through that
provider, and rerun `npm run verify:release`. Do not change the release status
to ready before that integration succeeds end to end.

Pre-existing `graphify-out/` changes were preserved and excluded from repair
commits.
