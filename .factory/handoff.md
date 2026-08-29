# Quarterly Ready — repair 8 handoff

## Outcome

Repaired the independent verification-8 P1 release blocker. The verifier's
exact command was first reproduced against candidate
`2611ee3c3238aa16603e0212e950b3ddf7e1116d`:

```sh
EXPECTED_BUILD_SHA=2611ee3c3238aa16603e0212e950b3ddf7e1116d npm run verify:live
# Error: safe entitlement fixture returned 404
```

The deployed Container App revision reported the candidate SHA but its active
template had only `PORT=8080`; it had dropped the required
`SAFE_QA_FIXTURES=1`. The source fixture was already deliberately constrained:
it accepts only one byte-for-byte bundled synthetic document, states
`charges: false` and `files_with_hmrc: false`, creates no billable entitlement,
and returns `fixture_only_no_filing` without calling Sociobot billing, Dodo,
HMRC, or an approved integration.

## Repair

- Kept the existing explicit deployment setting for `SAFE_QA_FIXTURES=1`.
- Made `scripts/deploy-container.sh` wait for the target build identity and
  then require a successful `/api/qa/entitlement` response with both safety
  declarations before it can report deployment success.
- Added deploy-contract regression coverage for the runtime setting and the
  post-deployment entitlement smoke check. This specifically prevents a
  healthy image with a missing runtime variable from being treated as a
  successful release.
- Updated the operations documentation so the release fixture and its safe
  non-filing policy match the enforced deployment behavior.

## Local verification

Run in a clean checkout on 2026-08-29:

```sh
npm ci
npm test
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
BUILD_SHA=repair-entitlement-qa cargo build --release
# every command declared in .factory/claims.json, separately and in order
```

Results:

- `npm ci`: 60 packages, 0 vulnerabilities.
- `npm test`: TypeScript typecheck; 9 Vitest tests; 13 Rust tests; deployment
  contract; production Vite build; 35 Chromium tests all passed.
- Browser coverage includes desktop, 390px mobile, keyboard navigation and
  dialog focus, Axe serious/critical checks on `/`, `/demo`, `/privacy`, and
  `/terms`, response errors, offline reload, privacy requests, and read/write
  rate limits with `Retry-After`.
- All 18 declared claims passed when each manifest command was run separately.
- Rust formatting and Clippy with warnings denied passed. The optimized Rust
  build produced `target/release/quarterly-ready` (12 MB).
- Frontend production build: JavaScript 41.02 kB (13.40 kB gzip), CSS 21.67 kB
  (5.33 kB gzip).

## Release verification

After the repair commit is pushed, deploy with:

```sh
./scripts/deploy-container.sh
EXPECTED_BUILD_SHA="$(git rev-parse HEAD)" npm run verify:live
```

The deploy command now fails unless the exact deployed build and safe
non-charging fixture are both live. `verify:live` then checks identity, both
Sociobot hosted-checkout paths, durable workspace storage, invalid input
rejection, the safe subscription-gated accountant-link and HMRC-submission
paths, designed 404 handling, and rate limits.

## Known gaps

Docker is not installed in this worker container, so a local Docker build could
not be run. The repository's deploy-contract regression and the Azure ACR
container build cover the deployment path; the live deployment evidence is
recorded after the command above completes.
