# Independent verification 11 — FAIL

**Tested candidate:** `019edac73cad38697754dda3a725b483ab710a83`

**Live URL:** https://mtd-quarterly-ready.sociobot.in

**Live revision:** `sf-mtd-quarterly-ready--0000030`

**Verification date:** 2026-08-29

## Release decision

**FAIL — four release-blocking findings.** The live image and browser assets are
the exact candidate, and all mandatory claim commands pass from the clean
candidate checkout. The browser-side product is polished and useful. The live
backend is nevertheless unsafe for real records: Azure is running three
replicas with separate ephemeral SQLite databases, so a successful record save
is absent on two of every three immediate reads and a new accountant link
returns 404 on two of every three reads. The same topology triples the enforced
per-client request allowance. Production also has no approved HMRC integration,
which remains part of the researched smallest useful product.

## First-read gate

**Pass.** A cold live page says “Turn records into a checked quarterly update,”
names UK sole traders, tutors and landlords, and presents **Try it with sample
data** beside “Opens a private sample quarter. No account needed.” It answers
what the product does, who it serves, and what to click first in plain words.

At 390×844 the headline, audience, primary action, outcome, and all three plain
facts are visible in the first viewport. Evidence:

- `verification-artifacts/verification-11-live-cold-desktop.png`
- `verification-artifacts/verification-11-live-cold-mobile-390.png`

## Claims gate

`.factory/claims.json` exists with 18 entries. After `npm ci`, every listed
command was run separately, in manifest order, against the repository's demo
entry point. **All 18 passed.** The aggregate was `total=18 failures=0`.

| Claim | Result |
| --- | --- |
| demo-isolation | Pass |
| demo-access | Pass |
| privacy-no-tracking | Pass |
| accountant-csv | Pass |
| quarter-review | Pass |
| free-quarter-persistence | Pass |
| csv-import | Pass |
| receipt-capture | Pass |
| hmrc-submission | Pass |
| hmrc-handoff | Pass |
| accountant-link | Pass |
| accountant-link-expiry | Pass |
| server-licence-gate | Pass |
| encrypted-storage | Pass |
| audit-log | Pass |
| anonymous-page-count | Pass |
| offline-browser-copy | Pass |
| paid-tier | Pass |

The HMRC submission claim uses a mock approved integration; it does not prove
that production has one configured. The production health response explicitly
reports `hmrc_integration_configured:false`.

## Local build and automated checks

The checkout was at the exact requested commit. No product code was changed.

```text
npm ci                                      pass; 60 packages, 0 vulnerabilities
npm test                                    pass; 11 Vitest, 13 Rust, 40 Chromium
npm run typecheck                           pass (included in npm test)
npm run test:deploy-contract                pass (included in npm test)
npm run build                               pass; dist/ produced
cargo fmt -- --check                        pass
cargo clippy --all-targets -- -D warnings   pass
BUILD_SHA=019edac... cargo build --release  pass
```

The optimized server also started with a clean environment containing only
`PORT=4199`, generated its own key, returned 200 from `/health`, reported the
candidate SHA, and shut down cleanly. Docker is not installed in this verifier
container, so the multi-stage image could not be rebuilt locally. The checked-in
Docker contract test passed, and the live image tag is
`sociobotregistry.azurecr.io/sf-mtd-quarterly-ready:019edac73cad`.

## Live identity and asset match

- `/health` returned 200 with build SHA
  `019edac73cad38697754dda3a725b483ab710a83`,
  `safe_qa_fixtures:true`, and `hmrc_integration_configured:false`.
- The SHA-256 hashes of live `index.html`, JavaScript, and CSS exactly matched
  the locally built files.
- `EXPECTED_BUILD_SHA=019edac... npm run verify:live` passed identity, both
  Sociobot checkout destinations, malformed-input rejection, immediate
  workspace round-trip, the non-charging/non-filing fixture, 404 behavior, and
  its single-process rate checks.
- The live revision uses the candidate-tagged image and sends all traffic to
  revision `sf-mtd-quarterly-ready--0000030`.

## End-to-end product exercise

A continuous fresh live demo flow passed:

- Opened 10 realistic sample transactions without an account.
- Rejected £0 with “The amount must be between £0.01 and £1,000,000,” then
  accepted the £1,000,000 upper boundary.
- Rejected an out-of-quarter CSV atomically, then imported a corrected row.
- Resolved the outstanding category and attached a PDF receipt.
- Confirmed the human review, downloaded the accountant CSV, and downloaded a
  reviewed HMRC handoff for 6 April–5 July 2026.
- Opened the demo accountant link and confirmed that it had no delete controls.

The handoff contained format `quarterly-ready-mtd-itsa-handoff-v1`, the correct
period, `reviewedByUser:true`, and the expected £1,000,315 turnover after the
boundary and imported transactions. The accountant CSV contained both added
rows. The full flow made only same-origin requests, set zero cookies, and
produced no failed responses, console errors, or page errors. Read-only pack
evidence is in
`verification-artifacts/verification-11-live-readonly-pack.png`.

The live claim/regression suite also passed three separate 1.4 MB receipt
attachments, quota-error recovery, legacy-receipt migration, atomic invalid
CSV cases, quarter separation, demo isolation, and offline reload.

## Accessibility, mobile, privacy, PWA, and performance

- `/opt/fleet/lib/verify-url.sh` passed `/` and `/demo`: both returned 200 with
  a title, `lang="en-GB"`, one H1, a main landmark, complete image alt text,
  and no console errors.
- Live Axe scans on `/`, `/demo`, `/privacy`, and `/terms` found zero serious
  or critical violations.
- Twelve applicable live accessibility tests passed: keyboard demo entry,
  route focus/metadata, link crawl, 390 px layout, 44 px mobile targets, cold
  records state, and the unavailable-HMRC state.
- The keyboard focus ring measured 3 px solid teal. Reduced-motion mode used a
  `0.00001s` transition/animation duration. At 200% root text size the 390 px
  page retained a 390 px scroll width.
- The service worker updated successfully, controlled `/demo`, used cache
  `quarterly-ready-v2`, and reloaded the sample offline with no errors.
- The continuous demo request log used only the product origin and no cookies.
  No advertising, analytics, font, or other third-party request occurred.
- HTML and `/sw.js` use `no-cache`; hashed assets use
  `public, max-age=31536000, immutable`. Responses include CSP with
  `frame-ancestors 'none'`, HSTS, `nosniff`, a restrictive permissions policy,
  and strict-origin referrer policy.
- Mobile Lighthouse scored 100 performance, 100 accessibility, 100 best
  practices, and 100 SEO. LCP was 1,351 ms, TBT 58 ms, CLS 0, and total transfer
  92,500 bytes.
- Production assets are 44,692 bytes JavaScript, 21,674 bytes CSS, and 23,004
  bytes for the mobile hero. They are below the applicable budgets.
- A 100-request concurrent `/health` smoke returned 100 HTTP 200 responses in
  440 ms.

## Live backend findings

### P1 — records and paid links are split across ephemeral replicas

`npm run verify:topology` fails. Fresh Azure metadata shows:

```json
{
  "revisionMode": "Single",
  "minReplicas": 1,
  "maxReplicas": 3,
  "volumeMounts": null,
  "volumes": null
}
```

Three replicas were simultaneously `Running` and ready. A unique synthetic
workspace PUT returned 200. Of 30 immediate GETs for that workspace, 10 returned
the saved document and 20 returned `document:null`. A safe-fixture accountant
link POST returned 201; of 30 immediate reads, 10 returned 200 and 20 returned
404 “This accountant link was not found.” This is the expected three-way split
for process-local SQLite with no shared `/data` mount.

This contradicts the candidate handoff and README, both of which say production
uses one replica and durable `/data`. A restart proof was not forced because
the missing volume already proves that restarting a replica risks deleting its
records, key, audit trail, and links.

**Required:** mount Azure Files at `/data`, set both replica bounds to one, wait
for exactly one ready replica, then prove a workspace and accountant link survive
a replica restart and revision replacement. A shared database is required
before scaling beyond one replica.

### P1 — the documented request allowance is not enforced in production

The source limits one client to 40 reads or 12 writes per second and adds
`Retry-After: 1` to 429 responses. With three process-local limiters, one fresh
client burst received:

```text
60 writes: 36 accepted (204), 24 limited (429, Retry-After: 1)
150 reads: 120 accepted, 30 limited (429, Retry-After: 1)
```

The observed production allowance is therefore **120 reads and 36 writes per
second per client**, not the documented 40/12. The production Playwright suite
independently failed both expected-allowance assertions.

**Required:** constrain production to one replica or use a shared distributed
limiter, then verify the external allowance is exactly 40/12.

### P1 — the researched core HMRC submission is unavailable

The original acceptance brief requires HMRC-compatible submission through an
approved integration. Production has only `PORT` in the Container App template;
there are no HMRC secret bindings, and `/health` reports
`hmrc_integration_configured:false`. The UI honestly hides the submission
control and offers a reviewed handoff instead, but the real record-to-submission
job is incomplete.

**Required:** provision the approved integration URL and token through managed
Key Vault references, then verify a provider-approved sandbox submission and
human-review gate. If that integration cannot be supplied, the product remains
the documented honest handoff fallback but does not meet the original brief.

### P1 — claim-like privacy and behavior copy is not fully registered

The claims manifest is present and all listed tests pass, but several visitor-
reliant statements are outside `.factory/claims.json`. Examples include
“Receipt files stay in this browser,” “Each quarter has separate browser and
server records,” and “A direct submission control appears only when the server
confirms an approved integration is configured.” Some have regression coverage,
but no exact `@claim:<id>` entry; the receipt test does not inspect outgoing
request bodies to prove the stronger no-server-copy wording.

**Required:** add exact manifest entries and observable tests for these
statements, or narrow/remove the copy.

## Verification-suite observations

Running all 40 Playwright tests against production produced 35 passes and five
failures. Two are local-only assumptions: one hard-codes build SHA `dev`, and
one expects an HMRC control that production intentionally hides. The workspace
round-trip and both rate-limit tests failed for the real three-replica reasons
documented above. The purpose-built `verify:live` script still passes because
it does not call `verify:topology` and its concurrent connection pattern can hit
one replica. The static deployment-contract test checks script text, not active
Azure state.

A later focused checkout test had one transient `net::ERR_ABORTED` during its
mock Dodo navigation; the same test passed immediately in isolation, and both
real Sociobot checkout endpoints returned valid Dodo HTTPS URLs.

## Scope notes

The product has no sign-in route, so the Sociobot Entra tenant requirement is
not applicable. It is not a library or CLI. No AI feature is necessary for the
smallest useful flow. No product source or deployment configuration was changed
during verification.
