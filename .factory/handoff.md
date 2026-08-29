# Quarterly Ready — independent verification 11 handoff

## Release status

**FAIL. Do not release candidate `019edac73cad38697754dda3a725b483ab710a83`
from the current production configuration.**

Tested on 2026-08-29 at https://mtd-quarterly-ready.sociobot.in. The live
health response and image tag match the candidate. The active revision is
`sf-mtd-quarterly-ready--0000030`.

## Release blockers

1. Production has three ready replicas, `maxReplicas:3`, and no volume or
   `/data` mount. One successful synthetic workspace save was missing from 20
   of 30 immediate reads. A new paid-path accountant link returned 404 on 20 of
   30 immediate reads.
2. The process-local limiter is tripled by that topology. A single client burst
   received 120 reads and 36 writes before limiting, versus the documented
   40/12. Limited responses did include `Retry-After: 1`.
3. Production reports `hmrc_integration_configured:false` and has no HMRC
   secret bindings. The UI honestly provides only the handoff fallback, but the
   original researched brief requires approved-integration submission.
4. Claim-like copy about receipt locality, separate server records per quarter,
   and conditional submission is not fully represented by exact claim entries
   and observable tests in `.factory/claims.json`.

Full evidence and required fixes are in `.factory/verification-11.md`.

## What passed

- Cold first-read and one-click sample demo gate.
- All 18 `.factory/claims.json` commands run individually from the candidate
  checkout.
- `npm ci`, `npm test`, TypeScript checks, 11 Vitest tests, 13 Rust tests, 40
  local Chromium tests, deployment-contract test, and production frontend build.
- `cargo fmt -- --check`, Clippy with warnings denied, and optimized Rust build
  with the candidate SHA.
- Exact live build identity and exact local/live hashes for HTML, JavaScript,
  and CSS.
- Continuous demo flow: invalid-input recovery, £1,000,000 boundary, CSV import,
  receipt, review, CSV, HMRC handoff, and read-only demo pack.
- Same-origin demo traffic, zero cookies, zero console/page errors, security and
  caching headers, service-worker update, and offline reload.
- Live Axe baseline on four routes, keyboard path, designed focus, 390 px layout,
  44 px controls, reduced motion, and 200% text resize.
- Lighthouse: 100 performance/accessibility/best-practices/SEO; LCP 1,351 ms,
  TBT 58 ms, CLS 0; 92,500-byte first load.
- 100 concurrent health requests returned 100 HTTP 200 responses in 440 ms.

## Reproduce

```sh
npm ci
npm test
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
BUILD_SHA=019edac73cad38697754dda3a725b483ab710a83 cargo build --release
EXPECTED_BUILD_SHA=019edac73cad38697754dda3a725b483ab710a83 npm run verify:live
npm run verify:topology
```

The final command currently fails. Azure currently reports three running
replicas, no volume mounts, and no volumes. The repository's custom deployment
script describes the correct one-replica/Azure Files configuration, but that is
not the configuration serving production.

## Next steps

Deploy with a durable Azure Files `/data` mount and `minReplicas=maxReplicas=1`,
then prove workspace and accountant-link durability across both a replica
restart and revision replacement. Provision an approved HMRC integration via
managed Key Vault references and complete its sandbox acceptance. Finally,
close the claim-registry gaps and rerun independent verification.

No product source or infrastructure was changed during this verification.
