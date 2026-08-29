# Quarterly Ready — repair 4 handoff

Work order: `mtd-quarterly-ready-repair-4`  
Completed: 2026-08-29  
Artifact: Rust/axum and Vite/TypeScript in one Azure Container App.

## Outcome

The deploy failure is repaired and the public service is live at
`https://mtd-quarterly-ready.sociobot.in`.

- Deployed build SHA: `6dd17fc318d420e432e0b75cd8558e82fb75bf4a`
- Active Azure revision: `sf-mtd-quarterly-ready--0000014`
- `/health`: `{"status":"ok","build_sha":"6dd17fc318d420e432e0b75cd8558e82fb75bf4a"}`
- Scale: exactly one minimum and maximum replica.
- Durable storage: Azure Files share `sf-mtd-quarterly-ready-data-v3`, mounted
  at `/data`; it contains the generated encryption key and encrypted database
  snapshot. The live service uses a local SQLite working file and streams a
  snapshot to Azure Files after every real workspace mutation. This avoids
  SQLite advisory-lock failures on SMB while preserving state across restarts.

The requested base candidate `5f7e90dd581a8d1dbb8c24fdc42ffc65a1d62e7f`
is preserved unchanged in Git history.

## Deployment failure reproduced and repaired

The initial candidate image built successfully, but its public deployment had
three contract defects: its custom hostname was left with a disabled TLS
binding, it allowed three replicas while both the limiter and SQLite were
single-process, and `/data` was ephemeral. Adding an Azure Files mount exposed
the underlying runtime failure: SQLite cannot reliably hold its database lock
on that SMB share. A second issue was that the first deployment wrapper emitted
escaped JSON, which Azure rejected.

`scripts/deploy-container.sh` is now the repository-owned work-order
deployment configuration. It builds with ACR build arguments, configures the
Azure Files share, applies the SNI certificate binding, and enforces one
replica. `scripts/check-deploy-contract.mjs` guards all four settings. The
backend now restores the encrypted SQLite snapshot from `/data` to local
storage at boot, uses a single local SQLite connection, serializes snapshot
writes, and streams snapshots without unsupported Azure Files metadata or
rename operations.

Focused regression coverage:

- `workspace_snapshot_is_restored_after_a_restart` proves snapshot restoration.
- `startup_migration_retries_transient_sqlite_locks` covers the prior lock
  classification.
- `test:deploy-contract` asserts Azure Files, `/data`, SNI, one replica, and
  build identity.
- Existing shared read/write limiter regressions still assert 40 reads and 12
  writes followed by 429 plus `Retry-After: 1`.

## Verification evidence

- Clean `npm ci && npm test`: passed. It ran typechecking, 4 Vitest tests, 10
  Rust tests, the deployment contract test, production Vite build, and all 28
  Playwright tests.
- `cargo fmt -- --check` and `cargo clippy --all-targets -- -D warnings`:
  passed.
- ACR run `chsh`: succeeded at `2026-08-29T02:26:51Z`. The source archive
  explicitly excluded `.git`; the Dockerfile used `rust:1-alpine`, built with
  `BUILD_SHA`, and started non-root on port 8080.
- Public `verify-url.sh`: GET 200; title present; `lang=en-GB`; one h1; main
  landmark; no missing image alt text; no browser console errors; 721 ms load.
- The full local browser suite covers desktop and keyboard operation, dialog
  focus, Axe serious/critical findings, 390 px mobile layout and 44 px targets,
  200% text, reduced motion, demo privacy, offline reload/service-worker
  update, CSV/HMRC/export paths, and no cold-load console failure.
- Public limiter proof after deployment: 40 accepted reads then 8 responses
  with HTTP 429 and `Retry-After: 1`; 12 accepted writes then 8 equivalent
  429 responses, each under one forwarded client identity.
- Public durability proof: workspace
  `98ef9e76-d49b-410b-8378-7db5f662ca01` was saved with marker
  `durability-98ef9e76-d49b-410b-8378-7db5f662ca01`; after an explicit restart
  of revision `0000013`, then again after deployment of revision `0000014`,
  the same marker was retrieved from `/api/workspace`.
  The durable snapshot on Azure Files was 32,768 bytes after the save.

## Run and deploy

```sh
npm ci
npm test
./scripts/deploy-container.sh
```

For the public identity and core backend contract, compare `/health` with the
image source SHA, then exercise `/api/workspace` with a new UUID and a stable
`X-Forwarded-For` header. The deployment wrapper is intentionally a brief
single-replica handoff during release; this prevents concurrent SQLite writers
while retaining workspace state.

## Known external dependency

The Sociobot billing controller checkout registration remains external to this
repository. At the time of this repair its `monthly` and `annual` checkout
URLs returned the controller's 404 `enabled factory product` response. No
payment provider identifier or credential has been added to this product.
