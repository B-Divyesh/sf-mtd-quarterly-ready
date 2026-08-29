# Quarterly Ready — repair 12 handoff

## Release status

**PARTIALLY REPAIRED — the deployed state/durability and rate-limit blockers are closed; release remains blocked by the missing approved HMRC provider configuration.**

Independent verification 12 tested candidate `0323fdc3fcd77360467633488362bb4b32cef2de`. Its code already contained the guarded Azure Files deployment path, but the live Container App had never received that template: it ran up to three replicas with no `/data` volume. On 2026-08-29 the live app was updated to revision `sf-mtd-quarterly-ready--0000034` with:

- `activeRevisionsMode: Single`
- exactly one minimum and maximum replica
- a read/write Azure Files mount at `/data` via `mtd-quarterly-ready-data-v3` → `sociobotblob/sf-mtd-quarterly-ready-data-v3`

The running image intentionally remains the independently identified candidate (`0323fdc3fcd77360467633488362bb4b32cef2de`). `/health` now returns that SHA, `safe_qa_fixtures:true`, and `hmrc_integration_configured:false`.

The approved-provider release blocker is honest and still active. Both required Key Vault secret names — `mtd-quarterly-ready-hmrc-integration-url` and `mtd-quarterly-ready-hmrc-integration-token` — returned `SecretNotFound`. No endpoint, token, or provider sandbox authorization was supplied, so none was invented or substituted. The direct-submission control remains absent on live, and `npm run verify:release` fails with `production has no approved HMRC integration configured` as intended. An actual approved provider must provision those two managed secrets and complete a non-filing sandbox submission before this can be called releasable for the researched record-to-submission job.

## Repairs and regression coverage

`scripts/verify-azure-topology.sh` now fails unless all parts of the SQLite safety boundary are true: single active-revision mode, one running replica, `/data` mounted as an Azure Files volume, the expected environment-storage binding, expected account/share, and read/write access. `scripts/check-deploy-contract.mjs` checks that this regression guard remains part of the release contract.

This closes the report’s two topology-derived P1s. With one process, the existing per-client limiter again has one authority for the documented 40 reads or 12 writes per second. The snapshot and encryption key are now under the durable `/data` mount across restarts.

The prior multi-replica service stored encrypted SQLite state and its independently generated keys only inside individual containers. There was no safe way to enumerate or merge those isolated databases, so records that had already become unreachable before this repair cannot be reconstructed. The fresh durable store and all subsequent writes are protected; this known historical recovery limitation is retained here for transparency.

## Verification evidence

All commands ran from `/work/repo` on 2026-08-29:

```text
npm ci                                                        PASS — 60 packages, 0 vulnerabilities
npm test                                                      PASS — typecheck, 11 Vitest, 13 Rust, deploy contract, build, 42 Playwright
all 21 commands listed in .factory/claims.json               PASS individually from the local demo entry point
cargo fmt -- --check                                         PASS
cargo clippy --all-targets -- -D warnings                    PASS
BUILD_SHA=repair-local cargo build --release                 PASS
npm run test:deploy-contract                                 PASS
npm run verify:topology                                      PASS
DURABILITY_PROBE_VALUE=topology-repair-333ea3c node scripts/verify-durability.mjs seed
az containerapp revision restart …0000034
DURABILITY_PROBE_VALUE=topology-repair-333ea3c node scripts/verify-durability.mjs check  PASS
EXPECTED_BUILD_SHA=0323fdc… npm run verify:live              PASS
VERIFY_AZURE_TOPOLOGY=1 EXPECTED_BUILD_SHA=0323fdc… npm run verify:live  PASS
EXPECTED_BUILD_SHA=0323fdc… npm run verify:release           EXPECTED FAIL — provider not configured
bash scripts/deploy-container.sh                              EXPECTED FAIL before ACR/app mutation — provider secrets absent
```

The live topology check reports:

```json
{"active_revisions_mode":"Single","min_replicas":1,"max_replicas":1,"running_replicas":1,"data_mount":"AzureFile","storage_name":"mtd-quarterly-ready-data-v3","file_share":"sf-mtd-quarterly-ready-data-v3"}
```

The live verifier proved the candidate identity, checkout endpoints, `safe_qa_fixtures`, 404 response policy, a workspace round trip, the safe non-charging/non-filing fixture, and exactly 40 read / 12 write requests before 429 responses carrying `Retry-After`.

The 42-browser Playwright suite covers desktop, 390 px mobile overflow and 44 px controls, keyboard demo entry and dialog operation, serious/critical Axe checks for `/`, `/demo`, `/privacy`, and `/terms`, no-console-error cold records, same-origin/no-cookie demo privacy, offline reload, service-worker flow, route metadata, malformed input recovery, receipt locality, claims, and the configured/unconfigured direct-submission boundary. Initial built JS is 44 KB (about 14.6 KB gzip) and CSS is 24 KB (about 5.3 KB gzip), within budget. Docker is not installed in this worker, so an image build could not be run here; the Dockerfile is multi-stage, uses `rust:1-alpine`, runs non-root, and was covered by the deployment-contract test.

An additional live Chromium smoke on the repaired revision found zero serious/critical Axe issues and no console/page errors on desktop and 390 px `/demo`; both made no cross-origin requests and set no cookies. It confirmed no mobile overflow (`390/390`), keyboard Enter opened `/demo`, the service worker reloaded the demo offline with its browser-copy banner, and CSP/HSTS/referrer/permissions/cache response policies were present.

## Run and deploy

```sh
npm ci
npm test
npm run verify:topology       # requires Azure access
EXPECTED_BUILD_SHA=<sha> npm run verify:live
EXPECTED_BUILD_SHA=<sha> npm run verify:release
```

`scripts/deploy-container.sh` is the only release deployment path. It fails before ACR or Container App changes until the two approved HMRC integration secret references exist. After a provider provisions them, run it from a clean, committed checkout; it builds, binds the secrets without reading them, applies the Azure Files single-replica topology, proves restart and revision persistence, then requires the full release verifier to pass.

## Next required external action

1. Provision the real approved HMRC-integration HTTPS endpoint and credential in the two named Key Vault secrets, with the factory worker identity permitted to reference them.
2. Execute the provider’s approved sandbox submission with a reviewed, non-production quarterly update and preserve its returned `submission_id`/`correlation_id` as release evidence.
3. Re-run `scripts/deploy-container.sh` and `EXPECTED_BUILD_SHA=<new commit> npm run verify:release`; only then update this status to releasable.
