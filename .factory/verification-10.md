# Independent verification 10 — FAIL

**Tested candidate:** `d60c79885edb2f5637e641ba0d193990b0099e24`

**Live URL:** https://mtd-quarterly-ready.sociobot.in

**Live revision:** `sf-mtd-quarterly-ready--0000025`

**Verification date:** 2026-08-29

## Release decision

**FAIL — four P1 release blockers.** The live service is the exact candidate and
the earlier safe-fixture deployment fault is repaired. However, the production
revision cannot perform a real HMRC submission, has no durable server volume,
may scale a process-local SQLite database and limiter to three replicas, and
cannot save a third valid near-limit receipt in a normal Chromium storage
quota. These failures affect the core record-to-submission job.

## First-read gate

**Pass.** A cold live page says “Turn records into a checked quarterly update,”
names UK sole traders, tutors and landlords, and shows **Try it with sample
data** beside “Opens a private sample quarter. No account needed.” The action is
one click from the landing page.

At 390×844, the headline, audience sentence, action, explanation, and all three
plain facts fit in the first viewport. The action occupies vertical pixels
526–571; the third fact ends at pixel 774. Desktop and mobile cold-page captures
are in `verification-artifacts/live-cold-desktop.png` and
`verification-artifacts/live-cold-mobile-390.png`.

## Claims gate

`.factory/claims.json` exists with 18 entries. After the required clean
`npm ci`, every listed command was run separately against the documented demo
entry point. All 18 passed:

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

The declared receipt claim covers one small PDF only. It does not cover the
cumulative storage failure described below. The declared HMRC claim uses a mock
approved integration; it does not prove that production has one configured.

## Local build and automated checks

All available repository gates passed from this candidate:

```text
npm ci                                      60 packages, 0 vulnerabilities
npm test                                    9 Vitest, 13 Rust, 36 Chromium
npm run typecheck                           pass (included in npm test)
npm run test:deploy-contract                pass (included in npm test)
npm run build                               pass; dist/ produced
cargo fmt -- --check                        pass
cargo clippy --all-targets -- -D warnings   pass
BUILD_SHA=d60c798... cargo build --release  pass
```

The production frontend is 41.02 kB JavaScript (13.40 kB gzip) and 21.67 kB
CSS (5.33 kB gzip). The hero is 51.72 kB. The live JavaScript and CSS SHA-256
hashes exactly match the locally built files. Docker CLI is unavailable in the
worker, so the Dockerfile was not rebuilt locally; the live ACR image tag and
health identity both identify the exact candidate.

The release binary also started successfully with an empty environment except
`PORT=4199`. It generated its key, served `/health`, and shut down cleanly.

## Live deployment and backend evidence

- `/health` returned 200 with build SHA `d60c79885edb2f5637e641ba0d193990b0099e24`
  and `safe_qa_fixtures:true`.
- `EXPECTED_BUILD_SHA=d60c798... npm run verify:live` passed. It proved both
  checkout endpoints, immediate workspace round-trip, the explicitly
  non-charging/non-filing safe fixture, 404 behavior, and input rejection.
- For one stable client key, 40 read requests were accepted and the next eight
  returned 429. Twelve writes were accepted and the next eight returned 429.
  Every 429 had `Retry-After: 1`.
- A concurrent 100-request `/health` smoke returned 100 HTTP 200 responses in
  409 ms.
- Azure reports live image
  `sociobotregistry.azurecr.io/sf-mtd-quarterly-ready:d60c79885edb`, one current
  replica, and revision health `Healthy`.
- Azure also reports only `PORT` in the container environment, `volumes:null`,
  `volumeMounts:null`, `minReplicas:1`, and `maxReplicas:3`.

## End-to-end browser evidence

In a fresh live demo context, the following independent flow passed before the
storage stress case:

- Opened the ten-record sample without an account.
- Rejected £0, then accepted the documented £1,000,000 boundary.
- Rejected a receipt over 1.5 MB, then recovered and saved without it.
- Rejected an invalid CSV atomically, then accepted a corrected quoted CSV.
- Resolved the outstanding category, confirmed human review, and downloaded a
  dated HMRC handoff JSON.
- Made and opened the demo accountant link; its pack was read-only.
- Observed only same-origin requests, zero cookies, zero console/page errors,
  and zero unexpected HTTP errors during that flow.

The 390 px demo had no horizontal overflow and no visible interactive target
under 44×44 CSS pixels. Keyboard navigation exposed a 3 px teal focus ring and
the skip link. Reduced-motion transition duration was `0.00001s`. Live Axe
scans of `/`, `/demo`, `/privacy`, and `/terms` found zero serious or critical
violations. The service worker updated to `/sw.js`, cache
`quarterly-ready-v2`, and reloaded the sample offline.

Mobile Lighthouse under its default throttled profile scored 99 performance,
100 accessibility, 100 best practices, and 100 SEO. LCP was 1,391 ms, TBT
111 ms, and CLS 0. Initial transfer was 117,411 bytes with no third-party
resources. Hashed JS/CSS responses are cached for one year immutable; `/sw.js`
and HTML use `no-cache`. Responses include CSP with `frame-ancestors 'none'`,
HSTS, `nosniff`, a restrictive permissions policy, and strict-origin referrer
policy.

## Defects

| Severity | Finding | Fresh evidence | Required resolution |
| --- | --- | --- | --- |
| P1 / release-blocking | Production cannot perform the paid HMRC submission promised by the brief and UI. | The live container has only `PORT`; it has neither `HMRC_INTEGRATION_URL` nor `HMRC_INTEGRATION_TOKEN`. `approved_integration_from_env()` requires both, and the real submission route returns 503 after licence verification when they are absent. The passing live fixture explicitly returns `fixture_only_no_filing`. | Configure a real approved MTD ITSA integration using managed secrets and verify a safe acceptance path that proves the production integration is configured. |
| P1 / release-blocking | Server records, encryption key, audit trail, and paid accountant links are stored only on ephemeral container storage. | Azure reports `volumes:null` and `volumeMounts:null`. The image writes these assets under `/data`. An immediate round-trip passes only while the current container survives; restart or revision replacement can erase them. | Mount durable storage at `/data`, then prove records, links, audit entries, and the key survive a replica restart and a revision replacement. |
| P1 / release-blocking | Production topology violates the single-process persistence and rate-limit design. | Azure reports `maxReplicas:3`; the source README says production must stay at one replica because SQLite and rate limits are process-local. The checked-in deploy-contract test passed only by inspecting script text and did not detect this live drift. | Set live maximum replicas to one, or move state and rate limiting to shared services. Add a live topology assertion to release verification. |
| P1 / release-blocking | Receipt capture fails after a few individually valid files with no recovery message. | In a fresh live demo, two 1,400,000-byte PDFs saved (local document 3,735,615 bytes). The third, still below the advertised 1.5 MB per-file limit, left the form open and row absent while emitting `QuotaExceededError: ... exceeded the quota`. | Store receipt blobs in IndexedDB/OPFS or a durable backend object store, catch quota errors, preserve the form state, and test multiple near-limit receipts. |

## Scope notes

This product has no sign-in route, so the Entra tenant check is not applicable.
It is not a library or CLI. No product source was modified during verification.
