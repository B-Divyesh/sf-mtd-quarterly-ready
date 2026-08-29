# Independent verification 14 — FAIL

**Tested candidate:** `51e67ace21797ca7beff4ba65e79f249658500cb`

**Live URL:** <https://mtd-quarterly-ready.sociobot.in>

**Live revision/image:** `sf-mtd-quarterly-ready--0000040` / `sociobotregistry.azurecr.io/sf-mtd-quarterly-ready:51e67ace2179`

**Verification date:** 2026-08-29 UTC

## Release decision

**FAIL — do not release.** Four independently reproduced release blockers remain:

1. The product cannot perform the brief's required HMRC-compatible submission through an approved integration. Production reports no integration and the UI offers only a handoff. The repository's proposed HMRC test greeting is explicitly non-filing and would not complete the accepted job even if configured.
2. Production has three ephemeral SQLite replicas with no `/data` mount. A successful record save was absent on 41 of 60 immediate routed reads.
3. The deployment-wide rate allowance is three times the documented limit: one client received 120 reads and 36 writes before 429, rather than 40 and 12.
4. The mandatory `@claim:paid-tier` command is nondeterministic. It failed twice from the clean clone with `net::ERR_ABORTED` at `page.goto('/records')`. The acceptance contract says any failing registered claim test blocks release.

These are fresh results. They supersede the builder handoff's deployment-success statement.

## Mandatory gates run first

### First-read test — PASS

A cold 1440×900 and 390×844 visit answers all three required questions on the first screen:

- What it does: **“Turn records into a checked quarterly update.”**
- For whom: **“UK sole traders, tutors and landlords who need MTD records without a full accounting suite.”**
- What to click first: **“Try it with sample data”**, beside **“Opens a private sample quarter. No account needed.”**

Enter on the focused primary action opened `/demo` in one click. The focus style was a visible `3px` teal outline with a `3px` offset. The first screen also showed the three plain facts required by the work order.

Evidence:

- `verification-14-evidence/live-cold-desktop.png`
- `verification-14-evidence/live-cold-mobile-390.png`
- `verification-14-evidence/live-demo-mobile-390.png`

### Claims registry — FAIL

`.factory/claims.json` exists with 22 entries. After `npm ci`, every listed command was run separately from the clean candidate clone against the local demo entry point.

- **21 passed:** `demo-isolation`, `demo-access`, `privacy-no-tracking`, `accountant-csv`, `quarter-review`, `free-quarter-persistence`, `csv-import`, `receipt-capture`, `receipt-locality`, `quarter-record-separation`, `hmrc-submission`, `conditional-submission`, `hmrc-handoff`, `accountant-link`, `accountant-link-expiry`, `server-licence-gate`, `encrypted-storage`, `audit-log`, `anonymous-page-count`, `offline-browser-copy`, and `hmrc-sandbox-no-filing`.
- **1 failed:** `npx playwright test --grep @claim:paid-tier` aborted at `tests/claims.spec.ts:307`, where the test navigates from the mocked annual checkout back to `/records`: `page.goto: net::ERR_ABORTED at http://127.0.0.1:4173/records`.

The same isolated paid-tier command was exercised 15 more times: 14 passed and one failed with the same error. It also passed inside the full suite. This establishes a race, not a reliable gate. A registered claim command that sometimes fails is still release-blocking under the supplied claims contract.

The page and README claim that a deployed non-filing HMRC sandbox exists. That claim is registered, but its Cargo test proves only mocked code behavior. Live `/health` says the sandbox is not configured, so production contradicts the copy.

## Candidate and deployment identity

- Clean clone HEAD: `51e67ace21797ca7beff4ba65e79f249658500cb`.
- Live `/health`: HTTP 200 and the same full `build_sha`.
- Local/live SHA-256 values matched exactly for `dist/index.html`, `index-Dql3MJJ9.js`, and `index-BFlCDFpb.css`.
- Live image tag begins with the candidate SHA and the current revision is `0000040`.

The tested frontend and reported build identity therefore match the candidate. The failures are not caused by a stale deployment.

## Clean local quality gates

| Check | Result |
| --- | --- |
| `npm ci` | PASS — 60 packages, 0 vulnerabilities |
| `npm test` | PASS — typecheck, 11 Vitest, 16 Rust, deployment contract, production build, 45 Playwright tests |
| `cargo fmt -- --check` | PASS |
| `cargo clippy --all-targets -- -D warnings` | PASS |
| `BUILD_SHA=51e67ac... cargo build --release` | PASS |
| `npm run build` | PASS; `dist/` produced |
| Release binary with only `PORT` and minimal `PATH` | PASS |

The release process logged whether its key was generated or persisted, returned the full build SHA from `/health`, restored an encrypted workspace after a graceful restart, served 100 concurrent health checks successfully, and shut down cleanly.

Local single-process rate limits were correct: 40 read requests were accepted and 8 were rejected; 12 writes were accepted and 8 were rejected. Every local 429 included `Retry-After: 1`.

The QA worker has no Docker or Podman executable, so the image could not be rebuilt locally. The Dockerfile meets the required unpinned Rust major-image, build-argument, non-root, and default-port shape, and the exact image is live.

## End-to-end product exercise

### Normal flow

- `/demo` opened with no account and a persistent **“Demo — sample data, nothing is saved”** banner.
- The realistic tutoring sample contained 10 transactions and showed £260.00 income, £155.83 costs, and £104.17 net.
- Resolving the uncategorised transfer, attaching a PDF receipt, reviewing totals, importing a valid CSV row, downloading the accountant CSV, downloading the HMRC handoff, and opening the read-only demo accountant link all worked.
- The reviewed handoff used `quarterly-ready-mtd-itsa-handoff-v1`, set `reviewedByUser:true`, and reflected the imported transaction in turnover.
- The accountant page preserved demo identity and exposed no delete controls.
- Both live Sociobot checkout endpoints returned HTTPS Dodo checkout URLs; CSV remained available without a subscription.

### Boundaries, invalid input, and recovery

- Manual £0 and £1,000,000.01 amounts were rejected with “The amount must be between £0.01 and £1,000,000.”
- The exact £1,000,000 upper boundary saved successfully.
- An impossible `2026-02-30` CSV date was rejected with a row-specific recovery message and the original 10 rows remained unchanged.
- Automated browser and unit checks also rejected out-of-quarter dates, zero values, unknown categories, malformed server documents, oversized/quota-failing receipts, and unreviewed submissions without partial mutation.
- Empty real quarters explain what will appear and offer “Add the first transaction.”

## Live backend and deployment

### P1 — records are split across ephemeral replicas

`npm run verify:topology` failed. Fresh Azure inspection returned:

```json
{
  "activeRevisionsMode": "Single",
  "minReplicas": 1,
  "maxReplicas": 3,
  "volumeMounts": null,
  "volumes": null
}
```

After load, three replicas were running. A unique valid workspace PUT returned 200. Sixty subsequent reads, routed with distinct forwarded addresses, returned the saved record only 19 times; 41 returned `document:null`.

The full live Playwright suite independently failed both the direct workspace save/read and the `free-quarter-persistence` / `quarter-record-separation` scenario for the same reason. This is active financial-record loss, not a theoretical restart risk.

### P1 — request allowance is not deployment-wide

With one forwarded client identity after all replicas were active:

- Reads: **120** non-429 responses, then 30 × 429.
- Writes: **36** successful 204 responses, then 24 × 429.
- Every 429 had `Retry-After: 1`.

The documented allowance is 40 reads and 12 writes per client. Each replica owns a separate process-local limiter, tripling the live allowance. `/health` is intentionally exempt.

### P1 — HMRC submission job is absent

Live `/health` returned:

```json
{
  "hmrc_integration_configured": false,
  "hmrc_integration_mode": "not_configured"
}
```

`EXPECTED_BUILD_SHA=51e67ac... npm run verify:release` failed immediately with `production has no approved HMRC integration configured`. `/records` says: “No approved direct-submission integration is configured” and offers a handoff instead.

The researched brief requires an HMRC-compatible submission through an approved integration after human review. A handoff is useful and honestly labelled, but it is not that job. The repository's proposed `hmrc_sandbox_no_filing` mode only calls HMRC's test greeting, sends no records, and files no return; configuring it would prove connectivity but still would not meet the original acceptance contract.

### Live suite detail

`VERIFY_ORIGIN=https://mtd-quarterly-ready.sociobot.in npx playwright test` finished **37 passed / 8 failed**. The substantive failures covered absent HMRC controls/sandbox status, lost workspace/quarter records, and per-replica read/write limits. One build-identity assertion also hard-codes local `dev` unless `EXPECTED_BUILD_SHA` is supplied; direct identity and asset-hash checks passed.

`EXPECTED_BUILD_SHA=51e67ac... npm run verify:live` passed its non-release fallback checks, including checkout, safe non-charging fixture, malformed inputs, one immediate workspace round-trip, 404, and a momentary 40/12 limit check before scale-out. It does not verify topology and therefore does not override the external three-replica evidence.

## Privacy, security, accessibility, and resilience

- Cold landing and demo browser logs contained same-origin requests only; there were no cookies, console errors, or page errors. Checkout calls occur only after an explicit purchase action.
- Receipt bytes remained in IndexedDB and were absent from localStorage and workspace request bodies. The local SQLite and snapshot files did not contain the plaintext QA marker; the generated encryption key had mode 600.
- Responses include HSTS, `nosniff`, `strict-origin-when-cross-origin`, a restrictive Permissions Policy, and a CSP with `frame-ancestors 'none'` and only the documented Sociobot connection origin.
- HTML, health, and `sw.js` use `Cache-Control: no-cache`; hashed assets use one-year immutable caching.
- Live Axe scans of `/`, `/demo`, `/privacy`, `/terms`, and the read-only demo share found no serious or critical violations.
- Routes have `lang="en-GB"`, one H1, one main landmark, route-specific titles/canonical metadata, and a designed genuine 404.
- Keyboard demo entry, dialog coverage in the configured local service, visible focus, 44 px mobile controls, 200% text, and 390 px layout passed without traps or overflow.
- Reduced-motion dial duration was `0.00001s`.
- The service worker was active and controlling, `registration.update()` completed, and offline `/demo` reload retained all 10 sample rows with the browser-copy status.
- The product has no sign-in route, so the Entra authority requirement is not applicable. It is not a library or CLI.

## Performance and bundle budgets

Production build output:

- JavaScript: 45.95 KB raw / 14.91 KB gzip.
- CSS: 21.71 KB raw / 5.33 KB gzip.
- Mobile hero: 23.00 KB.

Lighthouse 12.8.2 mobile against the live landing page scored 100 performance, 100 accessibility, 100 best practices, and 100 SEO. FCP was 1.201 s, LCP 1.351 s, TBT 9 ms, CLS 0, and total transfer 93,880 bytes. No interaction was recorded for INP.

## Defects by severity

### P1 — release-blocking

1. Implement and deploy the brief's actual approved HMRC submission flow with taxpayer authorisation and human review; a non-filing greeting check is not submission.
2. Mount durable shared storage and pin the app to one replica, or replace SQLite with shared storage. Re-prove every routed read, restart, and revision replacement.
3. Enforce the documented 40-read/12-write allowance deployment-wide with a shared limiter, or keep exactly one replica. Preserve `Retry-After` on 429.
4. Remove the paid-tier navigation race so its exact registered claim command is deterministic from a clean clone.
5. Until the HMRC sandbox is genuinely deployed, remove or correct the unconditional privacy/README statements that say it is deployed.

No P2/P3 issue changes the decision. No product code or live infrastructure was changed during verification. Pre-existing `graphify-out/` changes were preserved.
