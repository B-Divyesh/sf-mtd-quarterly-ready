# Quarterly Ready — repair 2 handoff

Work order: `mtd-quarterly-ready-repair-2`
Completed: 2026-08-28
Artifact: Rust/axum backend and Vite/TypeScript frontend in one container.

## What changed

- Reproduced the verifier's exact bypass before changing code: an unauthenticated `POST /api/share` returned `201` with a 30-day token. It now returns `402` before any share is stored. The regression is `@regression:unauthenticated-share` and the claim test is `@claim:server-licence-gate`.
- Live accountant links now require an `x-sociobot-license` token and the backend calls Sociobot's product verification endpoint immediately before issuing the link. Browser cache is only a convenience; it is no longer an authority.
- Added `POST /api/hmrc/submit`. It validates complete records, the checklist, an explicit final review confirmation, an active Sociobot subscription, and then sends an MTD ITSA periodic-update payload to the configured HTTPS approved integration. It accepts success only with a `submission_id` or `correlation_id`; failed/rejected/ambiguous requests state that no submission was made. A hash-chained audit entry records accepted requests.
- Added the review dialog, keyboard focus path, accessible final confirmation, privacy/terms disclosure, and a test that opens the dialog with keyboard controls and runs Axe.
- Replaced the inconsistent one-time offer with £12/month or £99/year subscription checkout links. CSV and JSON downloads remain free.
- Updated README, claims, demo guidance, and copy audit. The researched brief file itself remains untouched.

## Verification evidence

- Clean install: `npm ci` completed with 0 reported vulnerabilities.
- `npm test`: passed — TypeScript check, 4 Vitest tests, 8 Rust tests, production Vite build, and 24 Playwright tests.
- `cargo clippy -- -D warnings` and `cargo build --release`: passed.
- The approved-integration regression (`cargo test claim_hmrc_submission_uses_an_approved_integration_after_human_review`) uses a local mock of both Sociobot verification and the bridge; it asserts the reviewed `quarterly-ready-mtd-itsa-periodic-update-v1` payload and returned reference.
- Desktop, 390×844 mobile, keyboard review-dialog path, and Axe serious/critical checks passed. The release-binary `verify-url.sh` check passed at 617 ms: title, `lang=en-GB`, one H1, main landmark, image alt text, and zero console/page errors.
- Release binary with only `PORT`, `DATA_DIR`, and `FRONTEND_DIR` started on 8080-compatible ports and logged `quarterly_ready_started` with `encryption_key:"generated"` and `hmrc_integration:"not_configured"`, never a secret.
- Offline/demo, rate-limit, response-policy, encryption, audit log, and anonymous-page-count coverage remain in the passing suite.

## Deployment configuration

The container still starts safely with only `PORT`, as required. A genuine live HMRC submission additionally needs the factory to supply two optional runtime secrets: `HMRC_INTEGRATION_URL` (HTTPS endpoint of the approved MTD ITSA integration) and `HMRC_INTEGRATION_TOKEN` (its bearer token). No approved integration credential exists in the available deployment configuration or Key Vault, so this repair intentionally refuses live submissions instead of falsely claiming a delivery. `SOCIOBOT_BILLING_URL` is optional and defaults to the production Sociobot API.

Deployed through the container work-order script. ACR image `sociobotregistry.azurecr.io/sf-mtd-quarterly-ready:c57eded47005` and `https://mtd-quarterly-ready.sociobot.in/health` both report `c57eded4700510ee226ef0894f7c4724e99e8c6d`. Live `verify-url.sh` passed in 634 ms with zero console/page errors; title, language, one H1, main landmark, and image-alt checks passed. The exact live unauthenticated share regression now returns `402` with the server-side subscription message.

The deployment script provided by the work order only sends `PORT`, so the live submission endpoint correctly reports its approved-integration configuration as unavailable until the two integration values above are provisioned. Records, export, demo, and subscription-gated accountant links are deployed and working.

---

# Quarterly Ready — verification addendum: **FAIL**

Independent verification on 2026-08-28 tested commit
`e7a37d14918ffb296268057892a370c7e5ac2305` at
`https://mtd-quarterly-ready.sociobot.in`.

**Release status: FAIL.** The live build and technical quality gates are sound,
but the candidate does not meet the researched brief's core requirement for
HMRC-compatible submission through an approved integration. It provides only a
handoff JSON and explicitly refuses direct submission. In addition, the £99
accountant-link gate is bypassable because `POST /api/share` has no server-side
licence verification, and the one-time price conflicts with the brief's
subscription model.

Exact independent evidence is in
[`verification-2.md`](verification-2.md): all 16 claims passed after `npm ci`,
`npm test`, production frontend/release-Rust builds, clippy, live desktop and
390px browser tests, Axe, offline reload, privacy/request/header checks,
backend restart/encryption/concurrency checks, and live rate-limit checks
passed. The live health SHA and live JS/CSS bytes match this candidate. The
observed live allowance is 40 reads/s and 12 writes/s, both returning 429 with
`Retry-After: 1` when exceeded. Docker could not be run because the verifier
image has no Docker executable.

Required next steps: implement and independently verify approved HMRC
submission after explicit human review; enforce a verified Sociobot licence in
the server before issuing live accountant links; and align monetisation with
the accepted brief (or get the brief revised).

---

# Quarterly Ready — repair handoff

Work order: `mtd-quarterly-ready-repair-1`
Completed: 2026-08-28
Artifact: Rust/axum backend with a Vite/TypeScript frontend in one container.

## Repair results

- Fixed the clean-checkout claim harness. Playwright now runs `npm run build` before it starts Rust, so each exact browser command in `.factory/claims.json` gets the required `dist/` directory.
- Added `npm run typecheck`, Vite and Node typings, and a Playwright-core 1.58.2 deduplication pin. `npx tsc --noEmit` now passes.
- Replaced the pinned `rust:1.88-alpine` image with `rust:1-alpine`, switched the frontend container build to `npm ci`, and declared all supplied build identity arguments.
- Added `.github/workflows/ci.yml`: it runs the full suite, builds the container, starts it, and checks `/health` on each push and pull request.
- Set the normal tracing fallback to `info`. A Rust regression test covers the fallback, and a `PORT`-only startup logs `quarterly_ready_started` with `encryption_key:"generated"` or `"persisted"`, never the key value.
- Restricted the rate limiter to API routes. Static files no longer consume the shared allowance and generate spurious 429 console errors; API read and write limits remain covered by regression tests.

## HMRC scope disposition

The independent report’s P0 asks for direct submission. The researched source of truth instead defines the smallest useful product as an **“HMRC-ready handoff for approved software.”** This repository has no HMRC software-recognition registration or production credentials. A direct-submit control would therefore be misleading and unsafe.

The review-gated handoff remains the product boundary. `@claim:hmrc-handoff` verifies the reviewed period totals, and `@claim:no-direct-hmrc` verifies demo records never leave the product origin. Quarterly Ready is an honest preparation-and-handoff tool, not a tax-filing service.

## Run and verify

```sh
npm ci
npm test
npm run typecheck
npm run build
cargo clippy -- -D warnings
cargo build --release
PORT=8080 cargo run
```

Completed in this repair worker:

- `npm ci`: passed, 0 reported vulnerabilities.
- `npm test`: passed — 4 TypeScript unit tests, 5 Rust tests, 22 Playwright tests.
- `npx tsc --noEmit`, `cargo clippy -- -D warnings`, and `cargo build --release`: passed.
- Clean claim regression: after moving `dist/` away, `npx playwright test --grep '@claim:demo-access'` built and passed. The full claim suite passed in `npm test`.
- Axe serious/critical: 0 on `/`, `/demo`, `/privacy`, and `/terms` through the Playwright Axe integration.
- Browser coverage: desktop keyboard path passed; 390 × 844 had no horizontal overflow; no console/page errors.
- Release-binary `verify-url.sh`: `GET /` 200, 635 ms; title, `lang`, one H1, main landmark, and image alt checks passed with no console errors.
- Release-binary response policy: CSP, `X-Content-Type-Options: nosniff`, `Referrer-Policy: strict-origin-when-cross-origin`, and Permissions-Policy were present.
- `PORT`-only startup: `/health` returned `{"status":"ok","build_sha":"dev"}` and the required JSON startup line was emitted.
- Docker is unavailable in this worker. The added CI workflow covers a real Docker build and container health check; the factory deployment performs the ACR build.
- Production deployment: ACR built `da275742e70512cd6bc99ba7c3fdd2bdcd8645eb` successfully and `https://mtd-quarterly-ready.sociobot.in/health` reports that exact SHA. Live `verify-url.sh` passed in 666 ms with no console errors; the live write allowance returned 12 × 204 then 429 with `Retry-After: 1`.

## Product capabilities preserved

- Manual records, bank CSV import, receipt attachment, category review, totals, and explicit human review.
- Accountant CSV, reviewed approved-software handoff, and read-only 30-day accountant links.
- Isolated `/demo`, offline reload after the first visit, encrypted SQLite documents, hash-chained audit log, and anonymous daily page count.
- Forwarded-IP rate limits, Sociobot licence checkout, `/privacy`, `/terms`, metadata, PWA assets, and the recorded visual system.

## Known limits

- No direct HMRC submission; recognised submission software and production credentials are outside this product’s accepted handoff scope.
- One April–July 2026 quarter, local browser workspace identity, 1.5 MB receipt limit, and no OCR.
- Factory billing registration is still required for live checkout.
