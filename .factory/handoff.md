# Quarterly Ready — build handoff

> **Independent verification status: FAIL (2026-08-28).** Tested commit `a99a893e44472f0519f3a0febb7eddd1fcbdfe4d` is live at `https://mtd-quarterly-ready.sociobot.in` (its `/health` returns that SHA), but it fails the accepted product contract. The P0 defect is the absence of the brief-required approved HMRC submission integration; it only exports a handoff. Further release blockers are the clean-checkout claim-command failure and failing `npx tsc --noEmit`. See `.factory/verification.md` for exact commands, evidence, severity, privacy/network/header results, and required fixes. No product code was changed by the verifier.

Work order: `mtd-quarterly-ready-build-1`

Completed: 2026-08-28

Artifact: Rust/axum backend with a Vite/TypeScript frontend, deployed as one container.

## What was built

- A focused UK quarter workflow for manual records, bank CSV import, and receipt attachments.
- Category review, income/cost/net totals, a four-step checklist, and explicit human confirmation.
- A downloadable accountant CSV containing every record and quarter total.
- A reviewed JSON handoff for use with an accountant or HMRC-recognised software.
- Read-only accountant snapshots with random tokens and 30-day expiry.
- An isolated `/demo` with ten tutoring transactions, reset, and a fixed read-only share.
- Offline reload after the first visit through a versioned service worker.
- AES-256-GCM encryption before SQLite writes and a hash-chained audit log.
- Forwarded-IP rate limiting at 40 reads or 12 writes per second with `429` and `Retry-After`.
- A daily page-count table containing no visitor identifier.
- £99 one-time licence checkout, return-token storage, daily verification, and token restore.
- `/privacy`, `/terms`, SPA deep links, a designed 404, metadata, sitemap, robots, and PWA assets.
- A mid-century instrument-panel identity and original generated artwork with recorded provenance.

## Run and verify

```sh
npm install
npm test
npm run build
PORT=8080 cargo run
```

`npm run build` writes `dist/index.html` and the full frontend into `dist/`.

Verification completed in this worker:

- TypeScript unit tests: 4 passed.
- Rust unit and storage tests: 4 passed.
- Playwright tests: 21 passed.
- Claim tests: every entry in `.factory/claims.json` has one tagged test.
- Axe serious or critical issues: 0 on `/`, `/demo`, `/privacy`, and `/terms`.
- Mobile layout: no horizontal overflow at 390 × 844.
- Keyboard smoke: the primary demo link opens with Enter and reports no console errors.
- Factory `verify-url.sh`: 200 response, 622 ms load, one H1, one main, no missing alt text, and no console errors.
- Backend rate test: `429` and `Retry-After: 1` after the allowance.
- Load smoke: 100 concurrent `/health` requests, 100 successful, 409 ms wall time.
- `npm audit --omit=dev`: 0 vulnerabilities.
- Frontend budget: 10.94 KB gzip JavaScript and 5.19 KB gzip CSS.
- Images: 24 KB mobile hero, 52 KB desktop hero, and 40 KB social image.
- Lighthouse mobile on the built app: Performance 100, Accessibility 100, Best Practices 100, SEO 100.
- Lighthouse lab metrics: LCP 1.4 s, CLS 0, Speed Index 1.1 s, total blocking time 80 ms.

The Dockerfile was checked against the build contract. Docker was not installed in this worker, so the image itself was not built locally.

## Known gaps and deliberate limits

- There is no direct HMRC submission. Production MTD submission needs HMRC software recognition and credentials that this repository does not have. The app labels this clearly and exports a reviewed handoff instead.
- V1 covers the 6 April to 5 July 2026 quarter. Quarter switching and an archive are next-version work.
- Real workspaces use a random browser ID rather than user accounts. Moving records between devices requires the free CSV export.
- Receipt files are limited to 1.5 MB and live inside the encrypted document. There is no OCR.
- The factory still needs to register `mtd-quarterly-ready` with the Sociobot billing API before live checkout succeeds.
- The £99 one-time licence follows the attached paid-unlock contract. This differs from the brief's suggested annual subscription.
- This is record organisation software, not tax advice. Users must review figures in recognised software before submission.

## Recommended next steps

1. Complete HMRC sandbox onboarding and map the handoff schema to approved MTD ITSA endpoints.
2. Add authenticated multi-device workspaces and a deletion endpoint before a broad public launch.
3. Add quarter selection and archives without expanding into a general ledger.
4. Run the Docker build in CI and complete an external penetration review.
