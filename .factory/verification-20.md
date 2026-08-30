# Independent verification 20 — FAIL

**Verified:** 2026-08-30
**Candidate:** `5c6a3832b600a900e0e1d08034da91f2522eb713`
**Live URL:** <https://mtd-quarterly-ready.sociobot.in>

## Verdict: FAIL — do not release

The deployed page and static assets are from the candidate, but the deployed
backend is unsafe and does not meet the researched product contract. Fresh
live checks lost acknowledged financial records, did not enforce the required
rate limit, have no approved HMRC integration, and cannot start either paid
checkout. Azure topology explains the first two failures: the live service has
three running replicas, no `/data` volume, and a mutable image tag.

## First-read test — PASS

In a fresh cold browser context, the first screen says:

- **Does:** “Turn records into a checked quarterly update”.
- **For:** “UK sole traders, tutors and landlords who need MTD records without
  a full accounting suite.”
- **First action:** “Try it with sample data”, with “Opens a private sample
  quarter. No account needed.”

The visible action opens `/demo` without an account. The cold page had title
`Quarterly Ready — Check your MTD quarter`, `lang="en-GB"`, exactly one `h1`
and `main`, and no console/page errors. Evidence:
`.factory/verification-20-evidence/live-cold.json` and
`.factory/verification-20-evidence/live-cold-desktop.png`.

## Release-blocking defects

### Critical — 200-successful concurrent saves lose financial records

The fresh live `scripts/verify-concurrent-workspaces.mjs` probe made ten
concurrent `PUT /api/workspace` writes in its first round. Each write returned
200; after its prescribed wait, **six of ten** records were absent (indexes
`0,1,2,4,7,9`). A separate run during the live Playwright suite lost indexes
`2,3,7,9`. The normal real-quarter flow likewise saved a document and then
read `document: null`.

This is unrecoverable loss after acknowledgement, so the core record-to-quarter
job is unsafe. Evidence: `live-concurrency.log` and `live-playwright.log`.

### Critical — the documented API allowance is not enforced in production

The advertised allowance is 40 reads and 12 writes per client. With the
repository's stable-connection probe against live production:

- read 41 returned **400**, not 429;
- write 13 returned **204**, not 429;
- the OAuth callback after its write quota returned **400**, not 429.

Thus a client was admitted for at least 41 reads and 13 writes, and no positive
`Retry-After` was returned at the specified limit. This fails the mandatory
backend rate-limit contract and the `api-rate-limit` product claim. Evidence:
`live-rate-read.log`, `live-rate-write.log`, and `live-playwright.log`.

### Critical — live deployment violates the single-writer durable-storage contract

Fresh Azure queries reported `minReplicas: 1`, `maxReplicas: 3`, **three
running replicas**, no `volumeMounts`, no `volumes`, and mutable image
`sociobotregistry.azurecr.io/sf-mtd-quarterly-ready:5c6a3832b600`.
`npm run verify:release` consequently fails its topology check. This directly
explains process-local SQLite snapshots and quota state splitting between
replicas. It is not a source-build failure; it is a deployed runtime failure.

### Critical — no approved HMRC integration or taxpayer consent flow is live

Live `/health` from the requested candidate reports:

```json
{
  "hmrc_integration_configured": false,
  "hmrc_integration_mode": "not_configured",
  "hmrc_taxpayer_consent_required": false,
  "hmrc_provider_name": null
}
```

The product honestly hides direct submission, but the researched brief requires
HMRC-compatible submission through an approved integration after human review
and explicit taxpayer consent. `verify:release` cannot pass this requirement.

### High — both advertised subscription checkouts return 503

Fresh POST requests to both Sociobot checkout endpoints returned HTTP 503 HTML,
not a Dodo checkout URL:

- `/api/v1/products/mtd-quarterly-ready/checkout`
- `/api/v1/products/mtd-quarterly-ready-annual/checkout`

This blocks the £12/month and £99/year paid accountant-link path. Evidence:
the two `*-checkout.headers` and `*-checkout.body` artifacts; `npm run
verify:live` also stopped at the monthly 503.

## Passing evidence

- `.factory/claims.json` exists with 24 entries. **Before the wider QA suite,**
  every exact listed command was invoked individually from the clean install
  against its local demo entry point. All 24 passed: `demo-isolation`,
  `demo-access`, `privacy-no-tracking`, `accountant-csv`, `quarter-review`,
  `free-quarter-persistence`, `csv-import`, `receipt-capture`,
  `receipt-locality`, `quarter-record-separation`, `hmrc-submission`,
  `hmrc-consent-no-records`, `conditional-submission`, `hmrc-handoff`,
  `accountant-link`, `accountant-link-expiry`, `server-licence-gate`,
  `encrypted-storage`, `audit-log`, `anonymous-page-count`,
  `offline-browser-copy`, `paid-tier`, `hmrc-sandbox-no-filing`, and
  `api-rate-limit`.
- Clean local `npm test` passed: typecheck, 11 Vitest tests, 18 Rust tests,
  deploy contract, exact Vite production build, and 53 Playwright tests.
  `cargo fmt -- --check`, `cargo clippy --all-targets -- -D warnings`, and
  `BUILD_SHA=5c6a3832b600a900e0e1d08034da91f2522eb713 cargo build --release`
  also passed. The release binary served `/health` with only `PORT=4190` and
  generated its encryption key on first boot.
- Production build budgets pass: JS is 48.01 kB / **15.44 kB gzip** and CSS is
  21.71 kB / **5.33 kB gzip**. SHA-256 values of live JS/CSS exactly match the
  local build (`a71c93ad…b17ac` and `657cdda0…31afb`).
- Live desktop and 390 px scans of `/`, `/demo`, `/privacy`, and `/terms` had
  one `h1`, one `main`, no horizontal overflow, no console/page errors, and
  zero Axe serious/critical findings. `npm run verify:url -- <live>/demo`
  passed. Keyboard demo entry, reduced motion, 200% text, designed 404, and
  the live demo's normal capture/review/CSV/handoff/read-only-pack flows passed
  in the 44 passing live Playwright tests.
- Privacy: cold `/demo` made requests only to the product origin, had no
  cookies, and sent no third-party tracking requests. The local page-view call
  is same-origin. HTML/service worker used `no-cache`; hashed JS/CSS are
  `public, max-age=31536000, immutable`; responses include CSP with
  `frame-ancestors 'none'`, HSTS, `nosniff`, and strict-origin referrer policy.
- PWA: after service-worker update, a fresh live `/demo` reload offline showed
  “Offline — browser copy active” and “Maya Patel Tutoring”, with no errors.
  Evidence: `live-offline.json`.
- No sign-in path exists, so the Sociobot Entra External ID tenant requirement
  is not applicable.

## Live identity and test accounting

`/health` reports the requested commit
`5c6a3832b600a900e0e1d08034da91f2522eb713`; local and live static asset
hashes match. The full live Playwright run produced 44 passes, 1 skipped, and
8 failures. One failure is only its local-default assertion expecting
`build_sha: "dev"`; live correctly returned the requested SHA. The other seven
assertions are the persistence and rate-limit defects above.

`npm run verify:live` failed first at the monthly checkout 503.
`npm run verify:release` failed first at the required single-replica topology;
once topology is corrected it must also be rerun to prove the approved HMRC
capability.

## Required remediation and re-verification

1. Deploy exactly one active replica with `minReplicas=maxReplicas=1`, an
   Azure Files `/data` mount, and an immutable image digest; then prove two
   independent concurrent-save rounds preserve 20/20 acknowledged documents.
2. Make the limiter shared or retain the one-replica topology, and prove 429
   plus positive `Retry-After` precisely at read 41 and write 13, including
   the OAuth callback.
3. Restore/fix both registered Sociobot checkout endpoints.
4. Provision a real approved HMRC provider, approval reference, and taxpayer
   OAuth consent configuration; then rerun `npm run verify:release`.

The checkout and infrastructure changes require deployment/operator authority;
no product source was modified during this verification.
