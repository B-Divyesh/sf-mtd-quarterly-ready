# Independent verification 21 — FAIL

**Verified:** 2026-08-30

**Candidate:** `7c840e4853bbcb16270977bdb568271ebd86c746`

**Live URL:** <https://mtd-quarterly-ready.sociobot.in>

## Verdict: FAIL — do not release

The deployed frontend and backend match the candidate and the implemented
record-to-handoff workflow is healthy. Release is nevertheless blocked because
production has no approved HMRC integration and no taxpayer-consent flow. The
researched acceptance brief requires an HMRC-compatible submission through an
approved integration; the live product can only download a handoff for use in
other recognised software.

Fresh release-verifier evidence:

```text
Single revision; min/max replicas 1/1; one running replica; AzureFile /data;
immutable image sha256:bf4b906847a503f98dcf89f743faff366358bb169105020f5f759fb512ce98ca
Error: production has no approved HMRC integration configured
```

Live `/health` returned:

```json
{
  "status": "ok",
  "build_sha": "7c840e4853bbcb16270977bdb568271ebd86c746",
  "safe_qa_fixtures": true,
  "hmrc_integration_configured": false,
  "hmrc_integration_mode": "not_configured",
  "hmrc_taxpayer_consent_required": false,
  "hmrc_provider_name": null
}
```

## First-read gate — PASS

A cold 1440 × 900 visit clearly answers all three questions in the first
viewport:

- What it does: “Turn records into a checked quarterly update.”
- Who it is for: “UK sole traders, tutors and landlords who need MTD records
  without a full accounting suite.”
- What to click: “Try it with sample data,” followed by “Opens a private sample
  quarter. No account needed.”

The action opens `/demo` in one click. The resulting screen is already populated
with ten realistic transactions and shows the persistent “Demo — sample data,
nothing is saved” banner, Reset demo, and Start for real.

The cold page returned 200 with title `Quarterly Ready — Check your MTD quarter`,
one H1, no console/page errors, and only same-origin automatic requests.

## Release-blocking finding

### Critical — required live HMRC submission is absent

The live service reports `hmrc_integration_configured:false` and
`hmrc_integration_mode:"not_configured"`. It has no provider name and does not
require taxpayer consent. The records page correctly hides direct submission
and says that no approved integration is configured.

`EXPECTED_BUILD_SHA=7c840e... npm run verify:release` proved the repaired,
durable deployment topology, then failed at the mandatory approved-provider
assertion. The non-filing QA fixture and downloadable JSON handoff are useful
and honestly labelled, but neither submits an MTD quarterly update. This misses
the brief's core job-to-be-done and smallest-useful-product requirement.

Required remediation: provision a genuine approved MTD provider, approval
reference, and taxpayer OAuth configuration through the guarded deployment
path. Then prove consent, explicit human review, accepted submission, returned
reference, and audit persistence without using the non-filing QA fixture.

## Claims gate

`.factory/claims.json` exists with 24 entries. On the untouched checkout, the
first raw invocations occurred before dependencies were installed: all six Rust
claim commands passed, while 18 Playwright commands could not load
`@playwright/test`. After the required `npm ci` (60 packages, zero
vulnerabilities), every exact manifest command was rerun individually and all
24 passed:

| Claim | Result |
| --- | --- |
| `demo-isolation` | PASS |
| `demo-access` | PASS |
| `privacy-no-tracking` | PASS |
| `accountant-csv` | PASS |
| `quarter-review` | PASS |
| `free-quarter-persistence` | PASS |
| `csv-import` | PASS |
| `receipt-capture` | PASS |
| `receipt-locality` | PASS |
| `quarter-record-separation` | PASS |
| `hmrc-submission` | PASS (mock approved integration only) |
| `hmrc-consent-no-records` | PASS (mock provider only) |
| `conditional-submission` | PASS |
| `hmrc-handoff` | PASS |
| `accountant-link` | PASS |
| `accountant-link-expiry` | PASS |
| `server-licence-gate` | PASS |
| `encrypted-storage` | PASS |
| `audit-log` | PASS |
| `anonymous-page-count` | PASS |
| `offline-browser-copy` | PASS |
| `paid-tier` | PASS |
| `hmrc-sandbox-no-filing` | PASS |
| `api-rate-limit` | PASS |

The landing page, app, privacy/terms pages, README, demo guide, and claim
manifest were cross-checked. No material unlisted privacy or capability claim
was found. The mocked approved-provider claims do not establish that the live
provider exists; the conditional control correctly exposes that distinction.

## Clean local verification

- `npm ci`: PASS; 60 packages, zero audit vulnerabilities.
- `npm test`: PASS.
  - TypeScript typecheck: PASS.
  - Vitest: 11/11 PASS.
  - Rust: 18/18 PASS.
  - Deployment contract: PASS.
  - Vite production build: PASS.
  - Playwright: 54/54 PASS.
- `npm run build`: PASS and produced `dist/`.
- `cargo fmt --all -- --check`: PASS.
- `cargo clippy --all-targets --all-features -- -D warnings`: PASS.
- `npm audit --audit-level=high`: PASS; zero vulnerabilities.
- `BUILD_SHA=7c840e... cargo build --release`: PASS.
- Docker could not be rerun because this worker has no Docker command. The live
  topology check independently confirmed an immutable ACR image, one non-scaled
  replica, and the Azure Files `/data` mount.

The exact production build is 48.37 kB JavaScript (15.59 kB gzip) and 21.71 kB
CSS (5.33 kB gzip). The largest product image is 51.72 kB. These are comfortably
inside the stated budgets.

## End-to-end and invalid-input verification

The live suite passed 53 tests with one expected skip for a direct-origin
fallback that ingress makes inapplicable. Normal live flows covered sample demo
entry, category resolution, totals, transaction creation, CSV import, receipt
attachment, review, CSV download, HMRC handoff download, demo accountant link,
real quarter separation, persistence, and both checkout routes.

Observed sample outputs:

- Accountant CSV contained the sample business and all transactions (19 CRLF
  lines in the asserted export format).
- HMRC handoff used `quarterly-ready-mtd-itsa-handoff-v1`, period
  2026-04-06–2026-07-05, turnover £260, costs by category, and
  `reviewedByUser:true`.
- Invalid CSV dates, zero amounts, unknown categories, malformed records, and
  invalid quarter boundaries were rejected atomically.
- UI boundary recovery was explicit: £0 and £1,000,000.01 showed the allowed
  range; a 1,500,001-byte receipt showed the 1.5 MB limit; a subsequent
  £1,000,000 transaction saved successfully. The date input enforced
  2026-04-06 through 2026-07-05.
- Three valid 1.4 MB receipts persisted in IndexedDB without receipt bytes in
  localStorage or the server document. Simulated storage exhaustion preserved
  the transaction and announced how to recover.
- Unauthenticated live accountant-link creation returned 402. Both real
  monthly and annual checkout APIs returned a Dodo HTTPS checkout URL. No
  purchase was made.

## Backend, persistence, and rate limiting

- Candidate identity: `/health` reports the full requested SHA.
- Live concurrency: two rounds of ten concurrent saves preserved 20/20
  acknowledged documents after delayed reads.
- Live topology: Single revision mode, min/max 1/1, one running replica,
  private Azure Files `/data`, immutable image digest.
- A clean port-only release startup generated its key, reported the candidate
  build, and served 100/100 simultaneous health requests in 316 ms.
- A clean local restart then reported the key as persisted and restored the
  acknowledged encrypted workspace with 200. Rust coverage also verifies the
  durable snapshot, encrypted share, audit chain, and page-count schema.
- Live rate limit: 40 reads allowed; request 41 returned 429 with
  `Retry-After: 58`. Twelve writes allowed; request 13 returned 429 with
  `Retry-After: 57`. The OAuth callback shared the write allowance and also
  returned 429 with `Retry-After: 57` after twelve writes.
- Static assets did not consume API allowance; `/health` is exempt.

## Privacy, security headers, caching, and PWA

The full live demo flow recorded requests only to
`https://mtd-quarterly-ready.sociobot.in`, no cookies, no failed requests, and
no console/page errors. The only automatic endpoints were the same-origin app
shell and `/health`; the landing also sends the disclosed same-origin daily
page-count request. No advertising or analytics origin appeared. Explicit
checkout is the only tested user-triggered cross-origin path.

HTML and `/sw.js` use `Cache-Control: no-cache`; hashed JS/CSS use
`public, max-age=31536000, immutable`. Responses include HSTS, `nosniff`,
strict-origin referrer policy, camera/microphone/geolocation denial, and a CSP
with `frame-ancestors 'none'` and only self plus the Sociobot API where needed.

The live service worker hash matches the candidate. A fresh context updated the
worker, went offline, reloaded `/demo`, and showed both “Offline — browser copy
active” and the Maya Patel sample without errors.

## Accessibility, responsive behavior, and performance

- Desktop 1440 × 900 and mobile 390 × 844 were inspected.
- Mobile document width was exactly 390 px with no horizontal overflow.
- Each checked route had `lang="en-GB"`, a route-specific title, one H1, one
  main landmark, valid image alternatives, ordered headings, and a skip link.
- Keyboard-only demo entry and the review dialog passed; a fresh keyboard Tab
  produced the designed 3 px teal focus outline with 3 px offset.
- Mobile navigation, footer, and review controls met 44 px targets. The first
  action remained usable at 200% text.
- Reduced-motion mode made the dial update effectively instant.
- Axe reported zero serious or critical findings on `/`, `/demo`, `/privacy`,
  and `/terms`, including the review dialog.
- Lighthouse mobile `/demo`: performance 98, accessibility 100, best practices
  100, SEO 100; FCP 1.30 s, LCP 1.50 s, TBT 147 ms, CLS 0, 73,239 bytes.
- The designed unknown route returned a genuine 404. All internal links,
  `robots.txt`, and `sitemap.xml` returned expected statuses.

No sign-in path exists, so the Sociobot Microsoft Entra External ID requirement
is not applicable.

## Deployment parity

Live `/health` matches candidate `7c840e4853bbcb16270977bdb568271ebd86c746`.
Local and live SHA-256 hashes match for HTML, JavaScript, CSS, and the service
worker:

```text
index.html  3d98f41c2eeb28cea4421e3141e3fa2143de405b7ab50031581b5e5433adc463
JS          ab3c67b85725b5264855706119319cc3add2ed47f65cfd14f6cd312d6faca4b4
CSS         657cdda006edfa2359269c216a6bdd41bc10331d36e5e9d00675ee1529313afb
sw.js       70ea4ec1a3aea6a3e5750e80bd0c520479c67652e780ba99eedd63d42f6a9f43
```

The earlier deployment durability, topology, rate-limit, and checkout failures
are fixed in this candidate. The fresh result is based on current evidence, not
the prior deployment report. The approved HMRC provider remains the sole
release blocker.
