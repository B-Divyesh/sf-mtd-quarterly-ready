# Independent verification 13 — FAIL

**Tested candidate:** `257689fd66124aaad27e7918b45d4681ce534c25`

**Live URL:** <https://mtd-quarterly-ready.sociobot.in>

**Verification date:** 2026-08-29

## Decision

**FAIL — do not release.** The repository builds and all registered claims pass locally, and the live frontend/backend identify as the candidate. The live deployment nevertheless loses access to real records across replicas, allows three times its documented per-client request allowance, and has no approved HMRC submission integration. Those are three release-blocking defects in the researched job.

## First-read gate — PASS

A cold unauthenticated visit says **“Turn records into a checked quarterly update”**, names **UK sole traders, tutors and landlords**, and presents **“Try it with sample data”** beside **“Opens a private sample quarter. No account needed.”** At 390 px, all three facts are visible: demo isolation, CSV download, and read-only-link pricing. One keyboard Tab focused the sample action with a 3 px teal outline; Enter opened `/demo`.

Evidence:

- `verification-13-evidence/live-cold-desktop.png`
- `verification-13-evidence/live-cold-mobile-390.png`
- `verification-13-evidence/live-demo-mobile-390.png`

## Candidate and deployment identity

- Clean-clone HEAD and `origin/main`: `257689fd66124aaad27e7918b45d4681ce534c25`.
- Live `/health`: HTTP 200 with `build_sha` equal to that full SHA, `safe_qa_fixtures:true`, and `hmrc_integration_configured:false`.
- Azure Container App image: `sociobotregistry.azurecr.io/sf-mtd-quarterly-ready:257689fd6612`, revision `sf-mtd-quarterly-ready--0000035`.
- SHA-256 values for live and local `index.html`, JavaScript, and CSS matched exactly.

## Required claims gate — PASS locally

`.factory/claims.json` exists with 21 entries. In a separate clean clone at `/tmp/quarterly-ready-verification-13.w7R50n`, `npm ci` installed 60 packages with zero reported vulnerabilities. Every listed claim command was then run individually against the local demo entry point; **21/21 passed**:

`demo-isolation`, `demo-access`, `privacy-no-tracking`, `accountant-csv`, `quarter-review`, `free-quarter-persistence`, `csv-import`, `receipt-capture`, `receipt-locality`, `quarter-record-separation`, `hmrc-submission`, `conditional-submission`, `hmrc-handoff`, `accountant-link`, `accountant-link-expiry`, `server-licence-gate`, `encrypted-storage`, `audit-log`, `anonymous-page-count`, `offline-browser-copy`, and `paid-tier`.

The landing page, README, privacy notice, and terms were cross-checked against the registry. No material unlisted user-facing product claim was found. The local `hmrc-submission` test uses the documented mock approved integration; it does not establish that production has one.

## Clean local quality gates — PASS

```text
npm ci                                                    PASS — 60 packages, 0 vulnerabilities
npm test                                                  PASS — typecheck, 11 Vitest, 13 Rust, deploy contract, build, 42 Playwright
cargo fmt -- --check                                      PASS
cargo clippy --all-targets -- -D warnings                 PASS
BUILD_SHA=257689fd... cargo build --release               PASS
npm run test:deploy-contract                              PASS
PORT=4188 ./target/release/quarterly-ready                PASS
```

The release executable was started with only `PORT` plus a minimal executable `PATH`. It generated its own encryption key, served the built frontend, returned the full candidate SHA from `/health`, and shut down cleanly. Docker/Podman are unavailable in this worker, so a local image build was not possible. The matching live asset hashes and live build identity verify the deployed artifact.

Production bundle sizes are within budget: JavaScript 44.69 KB raw / 14.62 KB gzip; CSS 21.67 KB raw / 5.33 KB gzip. `dist/` was produced.

## Live product checks

### Normal, boundary, and recovery paths

- The demo opened without an account and showed ten realistic tutoring transactions, totals of £260 income, £155.83 costs, and £104.17 net.
- Category resolution, bank CSV import, accountant CSV export, HMRC handoff JSON, demo accountant link, and receipt attachment worked.
- A £0 manual amount was rejected with “The amount must be between £0.01 and £1,000,000.” Correcting it to the £1,000,000 upper boundary saved successfully.
- After category resolution and human review, the handoff contained `quarterly-ready-mtd-itsa-handoff-v1`, dates 2026-04-06 to 2026-07-05, `reviewedByUser:true`, and the updated turnover.
- Invalid date, out-of-quarter, zero-value, and unknown-category CSV rows were rejected atomically in the live browser run. Receipt-quota recovery also passed.
- There were no console or page errors in cold desktop/mobile, demo, boundary-input, accessibility, or offline checks.

### Accessibility, mobile, keyboard, and motion

- Live Axe scans on `/`, `/demo`, `/privacy`, and `/terms` found zero serious or critical findings; the independent `/demo` scan found zero findings at any impact.
- Each route has `lang="en-GB"`, one H1, and one main landmark. Metadata changes with SPA navigation. Internal links returned successful pages, and an unknown route returned the designed page with HTTP 404.
- At 390×844, `scrollWidth === clientWidth === 390`; the header/footer targets and review control passed the 44 px tests.
- Keyboard traversal reached the unresolved category select, receipt input, delete controls, and “I checked these figures” checkbox. Controls showed a 3 px teal focus outline; Space checked the review control. No trap was observed.
- Under `prefers-reduced-motion: reduce`, motion durations were reduced to 0.00001 seconds.

### Privacy, headers, caching, and PWA behavior

- A cold landing-to-demo flow made requests only to `https://mtd-quarterly-ready.sociobot.in`; no cookies were set. The expected aborted page-view request occurred only when the network was deliberately disabled.
- HTML and service-worker responses use `Cache-Control: no-cache`; the hashed JavaScript uses `public, max-age=31536000, immutable`.
- Responses include HSTS, `nosniff`, `strict-origin-when-cross-origin`, a restrictive permissions policy, and a CSP with `frame-ancestors 'none'` and only the documented Sociobot API connect origin.
- The service worker was active and controlling, `registration.update()` completed, and `/demo` reloaded offline with “Offline — browser copy active” and all ten sample rows.
- The site does not require sign-in, so the Entra authority requirement is not applicable. It is not a library or CLI.

### Performance

A Lighthouse 12.8.2 mobile run against the live landing page scored **100 performance, 100 accessibility, 100 best practices, and 100 SEO**. Measured FCP was 1.200 s, LCP 1.395 s, TBT 20 ms, CLS 0, and total transfer was 92,542 bytes. The synthetic run had no interaction from which to report INP.

## Release-blocking defects

### P1 — live records are split across ephemeral replicas

`npm run verify:topology` fails. Fresh Azure inspection showed `activeRevisionsMode: Single`, but `minReplicas:1`, `maxReplicas:3`, **no `/data` volume mount**, and **no volumes**. Load caused all three replicas of revision `0000035` to run.

A fresh valid workspace PUT returned 200. Of 30 immediate GETs for the same workspace, only **10** returned the saved document and **20** returned `document:null`, all with HTTP 200. The live `free-quarter-persistence` / `quarter-record-separation` browser scenario failed the same way. Records, encrypted snapshots, encryption-key continuity, audit history, and live accountant links therefore depend on which ephemeral replica receives a request.

**Required:** deploy the candidate through its guarded deployment path with the registered Azure Files share mounted read/write at `/data` and exactly one replica, or replace SQLite/process-local state with shared storage. Re-prove persistence across routing, replica restart, and revision replacement.

### P1 — live per-client rate allowance is tripled

The repository documents and locally enforces 40 reads and 12 writes per client per second. With all three live replicas active, one fresh client received **120 non-429 read responses** before 30 rate-limited responses and **36 successful writes** before 24 rate-limited responses. Every 429 correctly included `Retry-After: 1`, but the aggregate allowance was three times the documented limit. The focused live read/write regression tests failed accordingly.

**Required:** keep one replica while the limiter is process-local, or use a shared limiter keyed by the first `X-Forwarded-For` hop. Re-test externally for exactly 40 reads and 12 writes followed by 429 plus `Retry-After`.

### P1 — the required approved HMRC submission is absent

The researched minimum product includes submission through an approved integration after human review. Live health reports `hmrc_integration_configured:false`. Both expected Key Vault secrets, `mtd-quarterly-ready-hmrc-integration-url` and `mtd-quarterly-ready-hmrc-integration-token`, returned `SecretNotFound`. `EXPECTED_BUILD_SHA=257689fd... npm run verify:release` fails with `production has no approved HMRC integration configured`.

The UI honestly offers a reviewed handoff instead, but that fallback does not complete the accepted record-to-submission job.

**Required:** provision a real approved-provider endpoint and credential as managed secret references, deploy them through the guarded path, and prove one reviewed provider-sandbox submission without filing a real return.

## Live-suite detail

`VERIFY_ORIGIN=https://mtd-quarterly-ready.sociobot.in npx playwright test` produced 35 passes and 7 failures. Six are direct evidence for the three blockers above: missing submission controls (2), lost workspace/quarter records (2), and incorrect read/write limits (2). The remaining failure is a local-only harness assertion that hard-codes `build_sha:"dev"`; live correctly returned the candidate SHA.

## Scope

No product code or live infrastructure was changed. Pre-existing `graphify-out/` worktree changes were preserved. Only this report, the verifier handoff update, and three screenshots are intended verifier artifacts.
