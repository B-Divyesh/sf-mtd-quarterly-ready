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

## Product capabilities preserved

- Manual records, bank CSV import, receipt attachment, category review, totals, and explicit human review.
- Accountant CSV, reviewed approved-software handoff, and read-only 30-day accountant links.
- Isolated `/demo`, offline reload after the first visit, encrypted SQLite documents, hash-chained audit log, and anonymous daily page count.
- Forwarded-IP rate limits, Sociobot licence checkout, `/privacy`, `/terms`, metadata, PWA assets, and the recorded visual system.

## Known limits

- No direct HMRC submission; recognised submission software and production credentials are outside this product’s accepted handoff scope.
- One April–July 2026 quarter, local browser workspace identity, 1.5 MB receipt limit, and no OCR.
- Factory billing registration is still required for live checkout.
