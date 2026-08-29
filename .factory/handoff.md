# Quarterly Ready — repair 14 handoff

## Deployment status

The repair is deployed to <https://mtd-quarterly-ready.sociobot.in>.

- Source/runtime build: `9a7bff37887634764a61909c88611381b7109b3b`
- Image: `sociobotregistry.azurecr.io/sf-mtd-quarterly-ready:9a7bff378876`
- Azure Container Apps revision: `sf-mtd-quarterly-ready--0000045`
- ACR build: `ch151`, succeeded at `2026-08-29T17:03:39Z`
- Deployment mode: explicit `handoff-only`; `/health` reports `safe_qa_fixtures: true`, `hmrc_integration_configured: false`, and `hmrc_integration_mode: "not_configured"`.

This is an honest handoff-only deployment, not an approved live HMRC-filing release. The non-filing placeholder integration was removed from deployment, copy, and release verification.

## Repairs made

1. **HMRC capability and copy.** The release script now defaults to fail-closed `approved` mode and requires only Key Vault references named `mtd-quarterly-ready-approved-hmrc-url` and `mtd-quarterly-ready-approved-hmrc-token`; it never reads or logs them. The explicit `handoff-only` mode has no integration binding. README, privacy, terms, and the in-product output state now say that direct submission appears only with an approved integration and otherwise offer the reviewed handoff. The stale fake sandbox provisioner was removed. Regression: `@regression:hmrc-copy`, `@regression:hmrc-capability`, and `@claim:conditional-submission`.
2. **Durable, single-process state.** SQLite remains on the container-local filesystem (Azure Files cannot safely host SQLite locks). Every serialized mutation copies a durable snapshot and encryption key to mounted `/data`. Production uses Azure Files `mtd-quarterly-ready-data-v3`, one minimum/maximum replica, and `activeRevisionsMode: Single`. The deployer explicitly waits for the old revision to have zero running replicas before any image or durability-probe replacement, preventing two local SQLite databases from writing the same snapshot. Regression: `workspace_snapshot_is_restored_after_a_restart`, `durable_snapshot_restores_key_records_links_audit_and_page_count`, deployment-contract assertion for the stop-before-start replacement, and the deployed restart/revision durability probes.
3. **Rate limiting.** The live deployment is one process and keys production clients from the first `X-Forwarded-For` hop. It enforces exactly 40 read requests or 12 write requests, then returns `429` and `Retry-After: 1`. A UUID browser-session fallback prevents local direct-origin page-view tests from sharing one `direct` bucket; that fallback test is intentionally skipped only through a public ingress, where `X-Forwarded-For` is correctly authoritative. Regression: `@regression:shared-read-limit`, `@regression:shared-write-limit`, and `@regression:anonymous-page-view-fallback`.
4. **Paid claim navigation race.** The paid-tier claim now uses fresh pages after checkout navigation instead of using a document being replaced by a cross-origin checkout. Regression: `@regression:paid-tier-checkout-navigation`; the exact paid-tier claim passed 20 consecutive local repetitions.

## Verification evidence

All commands ran on 2026-08-29 from this checkout after `npm ci` (60 packages, 0 vulnerabilities).

| Check | Result |
| --- | --- |
| `npm test` | PASS — TypeScript, 11 Vitest tests, 16 Rust tests, deployment contract, production Vite build, and 47 local Playwright tests. |
| `cargo fmt -- --check` | PASS |
| `cargo clippy --all-targets -- -D warnings` | PASS |
| `BUILD_SHA=9a7bff… cargo build --release` | PASS |
| `npx playwright test tests/claims.spec.ts --grep '@claim:paid-tier' --repeat-each=20` | PASS — 20/20. |
| ACR container build `ch151` | PASS — production multi-stage image built from the committed source. |
| `npm run verify:topology` | PASS — `Single`, min/max `1/1`, exactly one running replica, read/write Azure Files `/data`. |
| `EXPECTED_BUILD_SHA=9a7bff… VERIFY_AZURE_TOPOLOGY=1 node scripts/verify-live.mjs` | PASS — live build identity, durable workspace, 40/12 limits, non-charging/non-filing fixture, topology. |
| Live Playwright (`VERIFY_ORIGIN=https://mtd-quarterly-ready.sociobot.in`) | PASS — 46 passed, 1 documented ingress-only skip. Covers desktop, 390 px mobile, keyboard, Axe serious/critical, privacy requests/cookies, response policy, offline demo reload, service-worker update, billing navigation, records, and rate limits. |
| Lighthouse mobile, live URL | PASS — performance 100, accessibility 100, best practices 100, SEO 100; LCP 1.4 s, TBT 30 ms, CLS 0, 92 KiB transfer. |
| `EXPECTED_BUILD_SHA=9a7bff… npm run verify:release` | EXPECTED FAIL — `production has no approved HMRC integration configured`; the required fail-closed release gate works. |

The first full local browser run during an ACR compile had one Chromium `SIGSEGV` while creating a context. A complete rerun after the build was idle passed all 47 tests; the later live run passed all applicable checks.

## Remaining external prerequisite

No authorised HMRC/provider credentials or taxpayer authority were available in this work order. Existing Key Vault entries explicitly described a non-filing sandbox, so using them to claim a submission would have been false and unsafe. A live filing release still requires the product owner to provision the approved provider URL/token under the two Key Vault reference names above, confirm the provider's HMRC authority and taxpayer consent flow, deploy with `DEPLOYMENT_MODE=approved`, and pass `npm run verify:release` plus an authorised end-to-end filing check. Until then, the deployed product truthfully supports quarter checking, CSV/accountant exports, and a reviewed recognised-software handoff only.

## Run and deploy

```bash
npm ci
npm test
cargo clippy --all-targets -- -D warnings
BUILD_SHA=dev cargo build --release

# Requires approved Key Vault references; otherwise exits before changing Azure.
bash scripts/deploy-container.sh

# Explicit truthful fallback used for this repair deployment.
DEPLOYMENT_MODE=handoff-only bash scripts/deploy-container.sh
```

Pre-existing dirty `graphify-out/` files were left untouched.
