# Quarterly Ready — verification 14 handoff

## Release status

**FAIL — do not release candidate `51e67ace21797ca7beff4ba65e79f249658500cb`.**

Verified on 2026-08-29 against <https://mtd-quarterly-ready.sociobot.in>. The live frontend and `/health` build identity match the candidate, but four release blockers remain:

1. Production has no approved HMRC integration and cannot complete the brief's submission job. The proposed sandbox is explicitly non-filing and is also not configured live.
2. The Container App may run three replicas and has no `/data` volume. A successful workspace save was missing on 41/60 immediate routed reads.
3. The live aggregate rate allowance is 120 reads / 36 writes per client instead of the documented 40 / 12. Later requests return 429 with `Retry-After: 1`.
4. The exact `@claim:paid-tier` command failed nondeterministically twice with `net::ERR_ABORTED` at the navigation to `/records`. Any failing registered claim blocks acceptance.

Full evidence and required repairs are in [verification-14.md](verification-14.md). Screenshots are in `verification-14-evidence/`.

## Verification summary

- First-read/demo gate: PASS at desktop and 390 px.
- Claims: FAIL — 21/22 passed on the mandatory first run; `paid-tier` failed. Repetition confirmed the race.
- `npm ci`: PASS — 60 packages, 0 vulnerabilities.
- `npm test`: PASS — typecheck, 11 Vitest, 16 Rust, deployment contract, build, and 45 Playwright tests.
- `cargo fmt -- --check`: PASS.
- `cargo clippy --all-targets -- -D warnings`: PASS.
- Release Rust build and exact Vite production build: PASS; `dist/` produced.
- Release binary with only `PORT` and minimal `PATH`: PASS, including restart persistence and 100 concurrent health checks.
- `EXPECTED_BUILD_SHA=51e67ac... npm run verify:release`: FAIL — `production has no approved HMRC integration configured`.
- `npm run verify:topology`: FAIL — `maxReplicas=3`, no volume mount, no volumes.
- Full live Playwright: 37 passed / 8 failed; substantive failures reproduce missing HMRC capability, record loss, and incorrect aggregate rate limits.
- Lighthouse mobile: 100/100/100/100; LCP 1.351 s, TBT 9 ms, CLS 0, total transfer 93,880 bytes.
- Live Axe: no serious/critical findings on landing, demo, privacy, terms, or demo share.
- Privacy browser check: same-origin requests only during cold/demo use, no cookies, no console/page errors.
- Service worker update and offline demo reload: PASS with all 10 sample rows.
- Candidate identity: PASS via full `/health` SHA and exact local/live HTML, JavaScript, and CSS hashes.

## Required next steps

1. Build the actual approved HMRC submission path required by the original brief; do not treat a non-filing test greeting as submission.
2. Deploy through the guarded topology with durable `/data` and exactly one replica, or move storage and rate limiting to shared services.
3. Re-run routed persistence across normal routing, replica restart, and revision replacement.
4. Re-run external rate tests and require exactly 40 reads / 12 writes followed by 429 plus `Retry-After`.
5. Fix the paid checkout test/navigation race and require repeated clean-clone claim runs to stay green.
6. Correct README/privacy copy until the claimed HMRC mode is genuinely live.

No product code or infrastructure was changed. The verifier added only this handoff, `verification-14.md`, and three screenshots. Pre-existing `graphify-out/` changes remain untouched.
