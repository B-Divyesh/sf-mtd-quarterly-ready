# Quarterly Ready — repair 13 handoff

## Release status

The three verifier blockers from candidate `257689f` are repaired. The live
container now uses one process over durable Azure Files, enforces one shared
40-read/12-write allowance per forwarded client, and exposes an explicit
Key Vault-backed HMRC non-filing sandbox mode.

The artifact remains a Rust/axum backend serving the Vite/TypeScript web app
from one container on `PORT=8080`.

## Failure reproduced before repair

On 2026-08-29, before deployment changes:

- `npm run verify:topology` failed with `expected minReplicas=1 and maxReplicas=1`.
- Azure showed `maxReplicas:3`, no `/data` mount, no volumes, no application
  secrets, and only `PORT` in the template.
- `EXPECTED_BUILD_SHA=257689f... npm run verify:release` failed with
  `production has no approved HMRC integration configured`.
- The independent report recorded only 10/30 routed workspace reads restoring
  the saved document, plus aggregate allowances of 120 reads and 36 writes.

## Repairs

### Durable state and deployment-wide limits

- The guarded deploy path creates and mounts the registered Azure Files share
  at `/data`, uses `activeRevisionsMode=Single`, and pins both replica bounds
  to one. The process-local limiter is therefore deployment-wide.
- The database snapshot, AES-256-GCM key, encrypted workspaces, accountant
  links, audit log, and page counts all persist on the mounted share. Page-count
  writes now trigger the same durable snapshot path as other mutations.
- The durability probe seeds both an encrypted workspace and accountant link,
  then requires all 60 routed reads to restore them after a replica restart
  and after a revision replacement.
- The release probe sends concurrent requests and requires exactly 40 reads
  and 12 writes before 429 responses with `Retry-After`.

### HMRC non-filing sandbox

- `HMRC_INTEGRATION_MODE=hmrc_sandbox_no_filing` is a distinct runtime mode.
  It is valid only with HMRC's official test greeting endpoint.
- The reviewed MTD payload is validated locally. Only a GET readiness check is
  sent to the official HMRC test API; no records or attestation secret leave
  the server, and the response states `files_with_hmrc:false`.
- The endpoint and generated attestation exist only in Key Vault. Container
  Apps receives managed-identity `secretRef` bindings; no value is in Git,
  image layers, application metadata, or deployment output.
- The browser says “HMRC non-filing sandbox” before confirmation and reports
  that no return was filed. Privacy and terms copy match this boundary.

### Exact regression coverage

- Rust restores the encryption key, encrypted workspace, encrypted link,
  audit entry, and page count from a durable snapshot.
- Rust rejects any sandbox URL except the official HMRC test endpoint and
  asserts that the sandbox request contains neither records, an Authorization
  header, nor the attestation.
- Playwright covers configured/unconfigured submission controls, sandbox copy,
  exact read/write limits, `Retry-After`, and live build identity.
- Deployment contract tests require the volume, one-replica ceiling, managed
  Key Vault references, sandbox mode, restart/revision probes, and release gate.

## Verification evidence

Clean/local:

- `npm ci`: 60 packages, 0 vulnerabilities.
- `npm test`: typecheck; 11 Vitest; 16 Rust; deploy contract; production build;
  45 Playwright tests.
- All 22 commands in `.factory/claims.json`: 22/22 pass individually.
- `cargo fmt --check`: pass.
- `cargo clippy --all-targets -- -D warnings`: pass.
- `BUILD_SHA=$(git rev-parse HEAD) cargo build --release`: pass.
- Release binary with only `PORT` and a minimal `PATH`: started, generated its
  key, served `/health`, and stopped cleanly.
- Production output: JavaScript 45.95 KB raw / 14.91 KB gzip; CSS 21.67 KB raw
  / 5.33 KB gzip; `dist/` produced.

Live/container:

- `bash scripts/deploy-container.sh`: ACR build, durable mount, one-replica
  topology, restart proof, revision-replacement proof, and release verification
  all pass. `/health` matches the deployed Git commit.
- Topology output: `Single`, `minReplicas=1`, `maxReplicas=1`, one running
  replica, `/data`=`AzureFile`, expected storage/share, HMRC configuration from
  Key Vault references.
- Restart proof: 60/60 routed workspace/link reads pass.
- Revision-replacement proof: 60/60 routed workspace/link reads pass.
- Live policy: 40 reads and 12 writes accepted; the next eight in each burst
  return 429 and all include `Retry-After`.
- HMRC release probe: configured mode `hmrc_sandbox_no_filing`, reviewed
  synthetic payload accepted, non-charging, and `files_with_hmrc:false`.
- `VERIFY_ORIGIN=https://mtd-quarterly-ready.sociobot.in npx playwright test`:
  45/45 pass, including desktop, 390 px mobile, 200% text, reduced motion,
  keyboard dialog/focus, Axe,
  privacy, offline reload/update, response policy, 404, and identity.
- Factory URL verifier: HTTP 200, title/lang/main/alt checks pass, no console
  errors, measured load 631 ms.
- Lighthouse 13.4.1 mobile: performance 100, accessibility 100, best practices
  100, SEO 100; FCP 1.230 s, LCP 1.380 s, TBT 7 ms, CLS 0, 93,853 bytes.
- Response policy: HSTS, `nosniff`, restrictive CSP, permissions/referrer
  policies; HTML/service worker `no-cache`; hashed assets immutable for one year.

Committed browser evidence is in `.factory/repair-13-evidence/`:

- `screenshot-desktop.png`
- `screenshot-mobile.png`
- `verify.json`
- `lighthouse.json`

## Known boundary and next step

This approved release path is deliberately non-filing. It proves reviewed MTD
payload validation and HMRC test-API connectivity but cannot file a taxpayer's
return. Production filing still requires HMRC application approval, taxpayer
OAuth consent, fraud-prevention headers, and production endpoint credentials.
The reviewed handoff remains available for recognised filing software.

Do not raise the replica ceiling while SQLite and the limiter remain local to
one process. Move both data and limits to shared services before scaling out.
