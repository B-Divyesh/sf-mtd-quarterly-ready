# Quarterly Ready — independent verification 9 handoff

## Current release status: FAIL

Candidate `0c99c04bc67fbd49e2403b97290569bb80bba607` is **not releasable** at
https://mtd-quarterly-ready.sociobot.in. The live `/health` endpoint reports
that exact SHA, but the required safe entitlement fixture is disabled:

```sh
EXPECTED_BUILD_SHA=0c99c04bc67fbd49e2403b97290569bb80bba607 npm run verify:live
# Error: safe entitlement fixture returned 404
```

Direct evidence: `GET /api/qa/entitlement` returns HTTP 404 with
`{"error":"The safe QA fixture is not enabled."}`. This is a P1 deployment
failure because it prevents safe, non-charging verification of the paid
accountant-link and HMRC-submission paths.

## What passed locally

From the candidate checkout: `npm ci`; every one of the 18 commands in
`.factory/claims.json`; `npm test` (9 Vitest, 13 Rust, 35 Chromium);
`cargo fmt -- --check`; `cargo clippy --all-targets -- -D warnings`; and
`BUILD_SHA=0c99c04bc67fbd49e2403b97290569bb80bba607 cargo build --release`.

The candidate's production frontend bundle is 13.40 kB gzip JavaScript and
5.33 kB gzip CSS. Local tests cover normal and invalid data, recovery, demo
isolation, privacy, offline reload, mobile, keyboard, accessibility, storage,
and rate limiting.

## Verified live behaviour

The cold first screen plainly names the job, target users, and a one-click
sample-data demo. Live build identity is correct; no third-party demo requests,
cookies, console errors, or serious/critical Axe findings were observed.
Desktop and 390px mobile worked, reduced motion is respected, focus is visible,
and rate limits allow 40 reads / 12 writes per client before 429 responses with
`Retry-After: 1`. Security headers and immutable hashed-asset caching are live.

## Next step

Restore the deliberately constrained `SAFE_QA_FIXTURES=1` runtime setting (or
the equivalent production configuration), redeploy, then run the exact
`EXPECTED_BUILD_SHA=... npm run verify:live` command successfully before asking
for another release verification. Full evidence is in
`.factory/verification-9.md`.
