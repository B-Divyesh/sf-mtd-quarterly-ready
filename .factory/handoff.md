# Quarterly Ready — independent verification 12 handoff

## Release status

**FAIL — candidate `0323fdc3fcd77360467633488362bb4b32cef2de` is not releasable at <https://mtd-quarterly-ready.sociobot.in>.**

The live deployment matches the candidate exactly, but it has three active, state-splitting replicas with no durable `/data` mount. A newly saved workspace was returned on 8 of 30 reads and absent on 22. The same topology raises the external per-client allowance from 40/12 to 120/36 requests per second. In addition, no approved HMRC integration is configured, so the researched record-to-submission job is incomplete.

Read the exact evidence, passing checks, and required repairs in `.factory/verification-12.md`.

## What the verifier ran

```text
npm ci
every command in .factory/claims.json (21/21 pass locally)
npm test (11 Vitest, 13 Rust, 42 Playwright pass)
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
BUILD_SHA=0323fdc... cargo build --release
EXPECTED_BUILD_SHA=0323fdc... npm run verify:live
npm run verify:topology (fails: maxReplicas=3 and no AzureFile /data mount)
EXPECTED_BUILD_SHA=0323fdc... npm run verify:release (fails: no approved HMRC integration)
```

Fresh live browser checks also covered first-read/demo entry, 390 px mobile, keyboard and focus, reduced motion, Axe serious/critical findings, console and page errors, same-origin/no-cookie demo traffic, headers/caching, service worker update/offline reload, normal/invalid/boundary records, CSV recovery, and live concurrency/persistence/rate-limit behavior.

## Next steps

1. Give real records and links durable shared storage; before using a distributed store/limiter, set both replica bounds to one.
2. Re-run routing, restart, revision-replacement, and external 40/12 rate-limit checks after the storage/limiter repair.
3. Provision the approved HMRC integration through managed secrets, then prove a reviewed provider sandbox submission and run `npm run verify:release`.
