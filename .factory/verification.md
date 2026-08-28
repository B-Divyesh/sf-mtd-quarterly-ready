# Independent verification — FAIL

Verified on 2026-08-28 against commit `a99a893e44472f0519f3a0febb7eddd1fcbdfe4d` and `https://mtd-quarterly-ready.sociobot.in`.

## Release decision

**FAIL.** The candidate does not deliver the brief's required approved HMRC submission integration: it explicitly creates only a handoff file and says it does not submit to HMRC. It also has a clean-checkout claim-command failure and a failing available TypeScript check.

## Cold live-page read

The first screen says it turns records into a checked quarterly update, names UK sole traders, tutors, and landlords, and leads with **Try it with sample data**. That button opens `/demo` in one click with a persistent “Demo — sample data, nothing is saved” banner. This first-read and demo requirement passes.

`GET /health` on the live deployment returned `{"status":"ok","build_sha":"a99a893e44472f0519f3a0febb7eddd1fcbdfe4d"}`, so the tested deployment matches the candidate.

## Claims gate

`.factory/claims.json` exists and declares 16 claims.

The mandatory fresh-checkout execution exposed a harness defect: after `npm ci`, every listed browser command such as `npx playwright test --grep @claim:demo-access` failed because Playwright's web server serves `/work/repo/dist`, which does not exist until a separate `npm run build`. The test command recorded in `claims.json` is therefore not runnable from a clean checkout as required. Before dependency installation, the commands also failed with `ERR_MODULE_NOT_FOUND` for `@playwright/test`; that is expected dependency setup rather than a product defect.

After `npm run build`, `npx playwright test --grep @claim` passed all 10 tagged browser tests (covering 13 browser claims), and the three exact Cargo claim commands passed:

| Claim group | Result after build |
| --- | --- |
| demo isolation/access, no tracking, no direct HMRC | pass |
| accountant CSV, quarter review, CSV import, receipt capture | pass |
| HMRC handoff, accountant link, 30-day expiry | pass |
| encrypted storage, hash-chained audit log, anonymous page count | pass |
| offline browser copy, paid tier | pass |

## Quality and product exercise

- `npm ci`: pass, 0 reported vulnerabilities.
- `npm test`: pass — 4 Vitest, 4 Rust, 21 Playwright tests.
- `npm run build`: pass; initial JS 10.94 KB gzip and CSS 5.19 KB gzip.
- `cargo clippy -- -D warnings`: pass.
- `npx tsc --noEmit`: **fail**. It reports missing `ImportMeta.env`, conflicting `Page` types in `tests/accessibility.spec.ts`, and missing Node `Buffer` types in `tests/claims.spec.ts`.
- Live desktop and 390×844 mobile `/demo`: axe serious/critical findings 0; no console or page errors; keyboard focus visible; reduced-motion treatment loaded.
- Manual live boundary/recovery check: malformed CSV shows “The CSV needs date, description and amount columns.”; a £0.01 income CSV record imports. The automated end-to-end suite covers normal import, receipt, review, export, handoff, and read-only-share paths.
- PWA: the live `/demo` has an active controller at `/sw.js`; `registration.update()` completed, and the offline-reload claim passed locally.
- Local backend: a workspace write survived a server restart, and the marker string was absent from the SQLite file; 100 concurrent `/health` requests passed.

## Privacy, security, deployment, and backend evidence

Cold live `/` requests were only the same origin: document, first-party JS/CSS/image, and `POST /api/page-view`. The context had no cookies, console errors, or page errors. Live `/demo` likewise made only same-origin requests and axe found no serious/critical issues.

The live responses include CSP, `X-Content-Type-Options: nosniff`, `Referrer-Policy: strict-origin-when-cross-origin`, and Permissions-Policy. Hashed JS/CSS use `Cache-Control: public, max-age=31536000, immutable`.

Rate-limit verification on the live backend with one `X-Forwarded-For` address: 12 sequential `POST /api/page-view` requests returned 204; requests 13–16 returned **429** with `Retry-After: 1`. This matches the observed write allowance of 12 requests/second. Health is exempt.

Docker could not be exercised because this verifier image has no `docker` executable. The source Dockerfile also violates the supplied backend build contract by using `FROM rust:1.88-alpine` rather than an unpinned `rust:1-slim` or `rust:1-alpine`; treat this as a deployment risk even though the currently live health endpoint confirms this SHA is deployed.

## Defects

| Severity | Finding | Evidence / required resolution |
| --- | --- | --- |
| P0 | No approved HMRC submission integration | The brief requires HMRC-compatible submission through an approved integration. The UI and README explicitly say it does not submit directly and only exports JSON. Implement and independently test the approved integration, including human review before submission, or change the accepted scope. |
| P1 | Listed browser claim commands are not clean-checkout runnable | `npx playwright test --grep @claim:…` fails after `npm ci` when `dist/` is absent. Make each recorded command build/serve its demo entry point or make Playwright's web-server command build first. The claims skill makes this release-blocking. |
| P1 | Type check fails | `npx tsc --noEmit` reports four errors. Add the intended Vite/Node typings and resolve the `Page` type mismatch; make type-check a passing package script. |
| P1 | Dockerfile violates the required Rust base-image contract | `Dockerfile` pins `rust:1.88-alpine`. Use the specified unpinned Rust major image and run a real container build in CI. |
| P2 | Default startup does not emit the required configuration log line | Running with no `RUST_LOG` emitted Cargo's line only; the application's `info!` startup message is filtered out. Configure a useful default log filter so generated-versus-persisted configuration is logged at normal startup. |

No product code was changed during this verification.
