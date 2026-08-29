# Independent verification 6 — FAIL

Verified 2026-08-29 against candidate commit
`a6e37268ebc04e26247e3c8499fa002a12cbf307` and
`https://mtd-quarterly-ready.sociobot.in`.

## Decision

**FAIL — do not release.** The deployed service is the requested candidate and
the free record-to-handoff workflow is working, but both advertised paid
checkout paths return HTTP 404. A visitor therefore cannot buy the service
needed for live accountant links or approved-integration submission. This is a
release-blocking paid-unlock failure.

## First-read and demo gate — PASS

A cold desktop and 390 px live load makes the job, audience, and first action
plain within the first screen:

- **What:** “Turn records into a checked quarterly update.”
- **For whom:** UK sole traders, tutors, and landlords who need MTD records
  without a full accounting suite.
- **First action:** “Try it with sample data”, with “Opens a private sample
  quarter. No account needed.”

The one-click action opens `/demo` with a realistic tutoring quarter, a
persistent “Demo — sample data, nothing is saved” banner, **Reset demo**, and
**Start for real**.

## Claims gate — PASS locally; paid outcome FAILS live

`.factory/claims.json` exists and declares 17 claims. From the prescribed
checkout, after `npm ci`, I ran every declared `test` command separately.
All **17/17** passed:

| Claims | Result |
| --- | --- |
| demo-isolation, demo-access, privacy-no-tracking | PASS |
| accountant-csv, quarter-review, csv-import, receipt-capture | PASS |
| hmrc-submission, hmrc-handoff | PASS (submission uses the shipped mock integration) |
| accountant-link, accountant-link-expiry, server-licence-gate | PASS |
| encrypted-storage, audit-log, anonymous-page-count | PASS |
| offline-browser-copy, paid-tier | PASS locally |

The local `paid-tier` test proves only that UI links have the intended URLs;
it does not prove the externally observable checkout result. Fresh manual
redirect-disabled requests returned:

- `https://api.sociobot.in/api/v1/products/mtd-quarterly-ready/checkout?plan=monthly`
  — **404**
- `https://api.sociobot.in/api/v1/products/mtd-quarterly-ready/checkout?plan=annual`
  — **404**

`EXPECTED_BUILD_SHA=a6e37268ebc04e26247e3c8499fa002a12cbf307 npm run verify:live`
fails immediately with: `monthly checkout returned 404, expected a
hosted-checkout redirect`.

## Build, test, and code-quality evidence

- `npm ci`: PASS (60 packages, 0 reported vulnerabilities).
- `npm test`: PASS: TypeScript check, 4 Vitest tests, 10 Rust tests, deploy
  contract, production Vite build, and 28 Playwright tests.
- `npm run build`: PASS; `dist/` produced. Initial JS is 35.36 KB raw / 11.66
  KB gzip; CSS is 21.22 KB raw / 5.27 KB gzip, within the stated budget.
- `cargo fmt -- --check`: PASS.
- `cargo clippy --all-targets -- -D warnings`: PASS.
- `BUILD_SHA=a6e37268ebc04e26247e3c8499fa002a12cbf307 cargo build --release`: PASS.
- Docker CLI was unavailable in this verifier image, so no local container
  build was possible.

## Live deployment, backend, and representative flows

- `GET /health` returned `status: ok` and the exact candidate SHA
  `a6e37268ebc04e26247e3c8499fa002a12cbf307`; the live deployment matches the
  tested candidate.
- The live demo showed £260.00 income, £155.83 costs, £104.17 net, and one
  unresolved category. Resolving it, confirming review, and downloading the
  HMRC handoff produced period `2026-04-06` to `2026-07-05`, turnover £260,
  and `reviewedByUser: true`.
- A fresh workspace rejected a missing browser workspace ID with 400 and a
  useful recovery message. A valid record saved and was retrieved on the live
  service, proving the normal persistence round trip.
- Public rate limiting is enforced: 40 reads in one second were accepted
  (these requests correctly returned 400 because no workspace ID was supplied)
  and request 41 returned **429** with `Retry-After: 1`; 12 writes returned
  204 and write 13 returned **429** with `Retry-After: 1`.
- An unknown route returns the designed recovery page with genuine HTTP 404.

## Privacy, accessibility, PWA, and browser evidence

- During the live demo flow, Playwright observed only
  `https://mtd-quarterly-ready.sociobot.in` requests; there were no cookies,
  analytics, ads, or third-party calls. Demo storage contained only
  `demo:quarterly-ready:document`.
- Live `/demo` had no console or page errors. Axe found zero serious/critical
  violations. The root, demo, privacy, and terms routes each have one `main`,
  one `h1`, an `en-GB` language declaration, and route-specific titles.
- Keyboard activation opens the demo; designed focus is visible. At 390 px
  there is no horizontal overflow and tested header/footer targets are at least
  44 px high. At 200% text, no horizontal overflow occurred and the heading
  and Reset demo control remained available.
- The installed `sw.js` uses versioned cache `quarterly-ready-v2`; after it was
  ready, offline reload retained sample records. `registration.update()`
  completed without a waiting worker. Reduced motion changes the dial
  transition to 0.01 ms.
- Response headers include CSP with `frame-ancestors 'none'`,
  `X-Content-Type-Options: nosniff`, `Referrer-Policy`, and a restrictive
  `Permissions-Policy`. Hashed JS has `Cache-Control: public, max-age=31536000,
  immutable`.

## Defects by severity

| Severity | Finding | Evidence and required resolution |
| --- | --- | --- |
| P0 | Paid checkout is dead | Both monthly and annual Sociobot checkout endpoints return HTTP 404 instead of an HTTPS hosted-checkout redirect. Register/enable the production product and re-run `npm run verify:live`; then exercise purchase restoration, a live accountant link, and approved submission. |
| P1 | Workspace API accepts malformed transaction records | `PUT /api/workspace` with `{"document":{"transactions":[{"date":"not-a-date"}]}}` returned 200 and persisted the object. Validate each record’s date, description, amount, kind, category, and receipt bounds at the API edge; add failing-input regression tests. This is particularly important for financial-record integrity, even though the browser form prevents this input. |
| P2 | Paid claim test is insufficiently observable | `@claim:paid-tier` checks hrefs and free CSV availability but cannot detect an external checkout 404. Extend it (or the live verification gate it invokes) to assert a hosted-checkout redirect in an allowed non-spending test path. |

## Re-test threshold

Keep this candidate blocked until both plans redirect to a real hosted checkout
and server-side document validation rejects malformed records. Then repeat the
full claims manifest, `npm test`, `npm run verify:live` with the expected SHA,
and the paid live accountant-link/submission path using an authorised test
licence.
