# Independent verification 5 — FAIL

Verified on 2026-08-28 against candidate commit
`c57eded4700510ee226ef0894f7c4724e99e8c6d` and
`https://mtd-quarterly-ready.sociobot.in`.

## Decision

**FAIL. Do not release this candidate.** The candidate passes its local claims
and quality suites from the prescribed checkout path, and its free demo and
record workflow are useful. Production nevertheless fails three mandatory
release conditions:

1. Both advertised subscription checkout links return HTTP 404, so a new user
   cannot buy access to live accountant links or HMRC submission.
2. The deployed product API did not enforce its documented request allowance;
   it returned no 429 in either a 200-request read burst or an 80-request write
   burst from one asserted client identity.
3. `/health` identifies the live server as `fb8d5f2…`, not the candidate
   `c57eded…`. Candidate frontend bytes match production, but the required
   candidate deployment identity does not.

No product code was changed during verification.

## Mandatory first-read gate — PASS

A cold 1440×900 and 390×844 live load answers all three required questions on
the first screen:

- What: turn records into a checked quarterly update.
- For whom: UK sole traders, tutors, and landlords who need MTD records without
  a full accounting suite.
- First action: **Try it with sample data**, with the adjacent explanation that
  it opens a private sample quarter without an account.

The action is above the fold and opens `/demo` in one click. The destination
immediately shows ten realistic transactions and the persistent “Demo — sample
data, nothing is saved” banner with **Reset demo** and **Start for real**.

## Mandatory claims gate — commands PASS; paid claim false live

`.factory/claims.json` exists with 17 entries. After checking out the exact
candidate at `/work/repo` and running `npm ci`, every listed `test` command was
run separately and passed: **17/17 commands, 0 failures**.

| Claim | Result |
| --- | --- |
| `demo-isolation` | PASS |
| `demo-access` | PASS |
| `privacy-no-tracking` | PASS |
| `accountant-csv` | PASS |
| `quarter-review` | PASS |
| `csv-import` | PASS |
| `receipt-capture` | PASS |
| `hmrc-submission` | PASS against recorded/mock integrations |
| `hmrc-handoff` | PASS |
| `accountant-link` | PASS for the fixed demo fixture |
| `accountant-link-expiry` | PASS |
| `server-licence-gate` | PASS |
| `encrypted-storage` | PASS |
| `audit-log` | PASS |
| `anonymous-page-count` | PASS |
| `offline-browser-copy` | PASS |
| `paid-tier` | Automated command passes, but observable production claim FAILS |

The `paid-tier` test only checks that the page links to Sociobot; it does not
check that checkout opens. Fresh GETs to both advertised destinations returned
404 with `{"error":"enabled factory product","status":404}`:

- `.../checkout?plan=monthly` — 404
- `.../checkout?plan=annual` — 404

The verification endpoint itself works for an invalid token and is rate
limited: one prior request plus 29 of an 80-request burst returned 200, then 51
returned 429 with `Retry-After: 2`. Observed allowance: **30 requests per
two-second window**. The claim test therefore does not prove the claimed paid
outcome as required by the claims contract.

## Clean checkout, tests, and builds

- `npm ci`: PASS; 60 packages installed, 0 reported vulnerabilities.
- `npm test`: PASS; TypeScript, 4 Vitest tests, 8 Rust tests, production Vite
  build, and all 24 Playwright tests passed.
- `npm run typecheck`: PASS.
- `cargo clippy -- -D warnings`: PASS.
- `npm run build`: PASS; `dist/` produced.
- `BUILD_SHA=c57eded… cargo build --release`: PASS.
- Docker build: not run because this verifier image has no `docker` executable.

The Playwright configuration hard-codes `FRONTEND_DIR=/work/repo/dist`. Tests
pass at the factory's prescribed `/work/repo` path, but a second clean worktree
at `/tmp/quarterly-ready-c57` failed browser tests because it served the wrong
directory. This is a portability defect in the test harness and contradicts
the README's ordinary clone-and-run instructions.

## End-to-end product exercise

The candidate release server and live deployment produced the same results for
the free workflow:

- Demo opened with 10 transactions and totals of £260.00 income, £155.83 costs,
  and £104.17 net.
- A malformed CSV showed “The CSV needs date, description and amount columns,”
  after which a valid one-row CSV imported successfully.
- A zero amount showed a specific recovery message; £0.01 at the quarter's
  start boundary saved; a date before the quarter was rejected by native form
  validation.
- A receipt over 1.5 MB showed the documented recovery message; a small PDF
  then attached successfully.
- Resolving the uncategorised transfer, confirming the figures, and exporting
  both files worked. The combined CSV contained the original and imported rows;
  the handoff contained format `quarterly-ready-mtd-itsa-handoff-v1`, period
  2026-04-06 to 2026-07-05, human review, and the updated £315 turnover.
- The demo accountant link opened a read-only pack with no delete controls.
- A fresh real `/records` workspace saved a £0.01 transaction to the server and
  restored it after the browser copy was removed. Local release-server data
  also survived a process restart; a raw database scan did not contain the
  test business name or transaction description.
- Live unauthenticated and invalid-token calls to `/api/share` and
  `/api/hmrc/submit` correctly returned 402. The paid success path could not be
  exercised because both checkout plans are dead.

### Console defect

A cold live `/records` load correctly shows the empty state, but its expected
`GET /api/workspace` 404 appears in Chromium as
“Failed to load resource: the server responded with a status of 404”. This
violates the no-console-errors-on-load gate. Root, demo, privacy, terms, and the
exercised demo flow had no console or page errors.

## Backend, identity, limits, and headers

- Candidate local `/health`: `c57eded4700510ee226ef0894f7c4724e99e8c6d`.
- Live `/health`: `fb8d5f29b93709dfd508a0220cd752e151504088`.
- Candidate and live HTML, hashed JS, CSS, and service worker were byte-for-byte
  equal. Product source differs between those commits only in factory handoff
  and graph metadata, but the live build identity still does not equal the
  requested candidate.
- Local candidate allowance: 40 reads/s, then 429; 12 writes/s, then 429; both
  return `Retry-After: 1`.
- Live result using one `X-Forwarded-For` identity: 200 concurrent
  `GET /api/workspace` requests all returned 400 and no 429; 80 concurrent
  `POST /api/share` requests all returned 400 and no 429. **Observed live
  allowance: greater than 200 reads and greater than 80 writes in the burst,
  with no `Retry-After`.** This fails the mandatory backend limit contract.
- The release server handled the concurrent bursts without 5xx responses.
- Browser response headers include CSP with `frame-ancestors 'none'`,
  `X-Content-Type-Options: nosniff`, `Referrer-Policy`, and
  `Permissions-Policy`. Hashed JS/CSS use
  `Cache-Control: public, max-age=31536000, immutable`.
- An unknown route renders the designed not-found screen but responds HTTP 200,
  not 404.

## Privacy, PWA, accessibility, and performance

- During the complete demo flow, only the document and same-origin hashed
  assets were requested. No workspace, share, HMRC, billing, analytics, or
  cross-origin requests occurred; the browser had no cookies. Demo storage used
  only `demo:quarterly-ready:document`.
- Offline reload passed locally and live after service-worker readiness. The
  live worker was active at `/sw.js`; `registration.update()` completed with no
  waiting worker, and cache `quarterly-ready-v2` remained active.
- Axe found **0 serious/critical findings** on `/`, `/demo`, `/privacy`, and
  `/terms`, locally and live. Each had `lang=en-GB`, one `main`, one `h1`, and a
  route-specific title.
- Keyboard focus showed a 3 px teal outline with 3 px offset. Keyboard-only
  entry opened the demo. Reduced motion changed the dial transition to 0.01 ms.
- At 390 px there was no horizontal overflow. At simulated 200% root text size,
  the document still had no horizontal overflow and the H1, reset, and export
  controls remained present.
- Some visible targets are below the required 44×44 px: the wordmark is
  148×34, the mobile Demo navigation link 39×44, and footer links are about
  22 px high. This is not reported by Axe but fails the attached touch-target
  baseline.
- `verify-url.sh`: PASS in 740 ms with title, language, one H1, main landmark,
  image alt text, and zero root-page console errors.
- Lighthouse mobile: performance 100, accessibility 100, best practices 100,
  SEO 100; FCP 1.23 s, LCP 1.38 s, TBT 43 ms, CLS 0, transfer 82,374 bytes.
  Lab INP was not available.
- Initial JS is 35.37 KB raw / 11.66 KB gzip; CSS is 21.07 KB raw / 5.27 KB
  gzip. Both pass budget.
- The product has no sign-in flow, so Entra authority verification is not
  applicable.

## Defects by severity

| Severity | Finding | Required resolution |
| --- | --- | --- |
| P0 | Monthly and annual checkout URLs both return 404 | Enable/register the production Sociobot product and add a claim test that follows checkout far enough to prove a valid hosted checkout response. Then exercise purchase/restore, live accountant link, and approved submission end to end. |
| P0 | Live API request allowance is not enforced | Ensure all production instances use the client identity consistently and return 429 plus `Retry-After` after 40 reads/s or 12 writes/s. Re-test through the public ingress. |
| P0 | Live build identity is not the candidate | Deploy the exact candidate (or nominate the actual deployed commit) and require `/health` to equal the tested SHA. |
| P1 | Fresh real-records load emits a console 404 | Represent an empty workspace without a failed-resource console entry, and add a cold `/records` console regression. |
| P2 | Several mobile/footer targets are under 44×44 px | Increase the clickable box without relying only on text dimensions. |
| P2 | Unknown routes return HTTP 200 | Return a real 404 status while preserving the designed recovery page. |
| P2 | Playwright server uses an absolute checkout path | Resolve `FRONTEND_DIR` from the current checkout so `npm test` works outside `/work/repo`. |

## Re-test threshold

Do not reconsider release until checkout returns a working hosted checkout,
the live product API produces 429 with `Retry-After` at the documented limits,
and `/health` reports the candidate under test. Then repeat the paid live
accountant-link and approved-HMRC submission paths with a valid test purchase,
plus the full claims manifest and `npm test`.
