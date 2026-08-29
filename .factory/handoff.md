# Quarterly Ready — independent verification 13 handoff

## Release status

**FAIL — do not release candidate `257689fd66124aaad27e7918b45d4681ce534c25`.**

Tested on 2026-08-29 at <https://mtd-quarterly-ready.sociobot.in>. The live frontend and `/health` match the candidate, and the repository is locally healthy, but the deployed backend violates its persistence and rate-limit contracts and lacks the approved HMRC integration required by the researched brief.

Full evidence: [verification-13.md](verification-13.md).

## What passed

- First-read gate: clear job, audience, first action, and one-click sample demo on desktop and 390 px mobile.
- All 21 `.factory/claims.json` commands passed individually in a clean clone after `npm ci`.
- `npm test` passed: typecheck, 11 Vitest tests, 13 Rust tests, deploy contract, production build, and 42 local Playwright tests.
- `cargo fmt -- --check`, strict Clippy, and the candidate-stamped release build passed.
- The release executable started with only `PORT`, generated its key, and reported the candidate SHA.
- Live demo workflows, invalid-input recovery, downloads, local receipt storage, demo sharing, privacy request logging, offline reload/update, desktop/mobile layout, keyboard focus, reduced motion, and live Axe checks passed.
- Lighthouse mobile: 100 performance, 100 accessibility, 100 best practices, 100 SEO; LCP 1.395 s, TBT 20 ms, CLS 0.
- Live and local HTML/JS/CSS hashes match. Security headers and cache policy are present.

## Release blockers

1. **P1 — deployed state is ephemeral and replica-local.** Azure reports `minReplicas:1`, `maxReplicas:3`, no `/data` mount, and no volume. Under load, all three replicas ran. A valid saved workspace appeared in only 10 of 30 reads; 20 returned `document:null`. `npm run verify:topology` and the live quarter-persistence scenario fail.
2. **P1 — the external request allowance is tripled.** A single fresh client was allowed 120 reads and 36 writes, not the documented 40/12, before 429. The 429 responses did include `Retry-After: 1`.
3. **P1 — no approved HMRC integration.** `/health` reports `hmrc_integration_configured:false`; both expected Key Vault secrets are missing; `npm run verify:release` fails. The handoff download is honest but does not fulfil the required submission workflow.

## Reproduce

```sh
npm ci
npm test
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
BUILD_SHA=257689fd66124aaad27e7918b45d4681ce534c25 cargo build --release
EXPECTED_BUILD_SHA=257689fd66124aaad27e7918b45d4681ce534c25 npm run verify:live
npm run verify:topology
EXPECTED_BUILD_SHA=257689fd66124aaad27e7918b45d4681ce534c25 npm run verify:release
```

`verify:live` passes in a single burst and confirms identity, checkout, fixtures, validation, and headers. `verify:topology` fails on replica/volume state. `verify:release` fails on missing approved HMRC configuration. Under scaled load, the persistence and aggregate-limit failures reproduce as recorded in the full report.

## Required next steps

1. Restore the Azure Files `/data` mount and one-replica ceiling, or move state and rate limiting to shared services.
2. Prove records and accountant links across routing, restart, and revision replacement; prove exactly 40 reads / 12 writes per client followed by 429 and `Retry-After`.
3. Provision the real approved HMRC provider secrets and complete a reviewed, non-filing sandbox submission.
4. Re-run every claim, the full local suite, `verify:topology`, and `verify:release` before requesting another independent verification.

No product code or infrastructure was changed during verification. Pre-existing `graphify-out/` changes were left untouched.
