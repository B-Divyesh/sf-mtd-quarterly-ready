# Independent verification 4 — FAIL

Verified 2026-08-28 against candidate commit
`fb8d5f29b93709dfd508a0220cd752e151504088` and
`https://mtd-quarterly-ready.sociobot.in`.

## Decision

**FAIL.** This candidate cannot pass the mandatory clean-clone claims gate:
the configured Playwright server serves the hard-coded, absent
`/work/repo/dist` instead of the clean clone's `dist/`. Every browser claim
which requires the frontend therefore fails. This is release-blocking under
the claims contract. Separately, the deployed product still lacks a configured
approved HMRC integration, so the real job promised by the researched brief
cannot be completed.

No product code was changed in this verification.

## First read and deployment identity

A cold live desktop load of `/` returned 200. In plain words it says it turns
records into a checked quarterly update, identifies UK sole traders, tutors,
and landlords, and presents **Try it with sample data** with “Opens a private
sample quarter. No account needed.” One click opened `/demo`, whose persistent
banner says “Demo — sample data, nothing is saved.” This gate passes.

`GET /health` returned
`{"status":"ok","build_sha":"fb8d5f29b93709dfd508a0220cd752e151504088"}`.
The live JavaScript and CSS names and byte sizes matched a fresh local Vite
production build (`index-DScAhmUU.js`, 35,372 bytes; `index-DBPfy_jM.css`,
21,069 bytes).

## Mandatory claims gate — FAIL

`.factory/claims.json` exists and contains 17 claims. From a new clone of the
repository, detached at the candidate commit, I ran `npm ci` and then every
exact test command listed in that manifest. The five exact Rust claim commands
passed. `@claim:server-licence-gate` passed. The other 11 claims failed because
the Playwright `webServer` command builds the clone but launches Rust with
`FRONTEND_DIR=/work/repo/dist`. That path does not exist in a clean clone, so
the browser receives no application frontend; for example
`@claim:demo-isolation`, `@claim:demo-access`, and
`@claim:privacy-no-tracking` time out looking for the mandatory demo banner.

The same defect makes the full clean-clone suite fail: `npm test` completed
with **8 passed, 16 failed** Playwright tests (exit 1). It includes all failed
browser claims, the accessibility browser checks, keyboard path, and static
asset regression. It is not acceptable to rely on a pre-existing workspace's
`dist/` directory; the acceptance contract requires the exact commands to work
from a clean clone through the demo entry point.

## Local checks

- `npm ci`: passed, 0 reported vulnerabilities.
- `npm run typecheck`: passed.
- `npm run test:unit`: passed (4 tests).
- `cargo test`: passed (8 tests), including all five Rust claim tests.
- `cargo clippy --all-targets -- -D warnings`: passed.
- `npm run build`: passed and produced `dist/`; initial JS was 11.66 KB gzip
  and CSS 5.27 KB gzip, within budget.
- `BUILD_SHA=fb8d5f29b93709dfd508a0220cd752e151504088 cargo build --release`:
  passed (the release binary was produced). Docker was unavailable in this
  verifier environment (`docker: command not found`).

## Live product, privacy, accessibility, and backend evidence

- Live `/demo` works in a fresh browser: it stores only
  `demo:quarterly-ready:document`, has no cookies, and its requests were only
  same-origin document/assets plus `POST /api/page-view` from the landing
  page. There were no console or page errors during the exercised demo flow.
- Representative demo exercise passed: bad CSV gives “The CSV needs date,
  description and amount columns”; resolving the unmatched transfer, attaching
  a PDF receipt, checking figures, CSV export (19 CRLF-separated lines), HMRC
  handoff download, and read-only demo accountant pack all worked. The handoff
  had the stated 2026-04-06–2026-07-05 period and £260 turnover.
- Live Axe found zero serious/critical violations on `/`, `/demo`, `/privacy`,
  and `/terms`; each has one `h1`, one `main`, and `lang=en-GB`. At 390px the
  demo had `scrollWidth == clientWidth == 390`; reduced-motion produced no
  running animations. Keyboard focus has a visible 3px teal outline and Enter
  opens the demo. Offline reload after service-worker readiness showed both
  “Offline — browser copy active” and the sample records.
- HTML/assets returned CSP with `frame-ancestors 'none'`, `nosniff`,
  Referrer-Policy, and Permissions-Policy. Hashed JS and CSS have immutable
  one-year caching. The response policy had no browser CSP errors.
- Live write rate limit was independently exercised with one
  `X-Forwarded-For` value: 12 `POST /api/page-view` requests returned 204;
  requests 13–16 returned 429 with `Retry-After: 1`. Observed write allowance:
  **12 requests/second**. `/health` is exempt.

## Release-blocking defects

| Severity | Finding | Evidence and required resolution |
| --- | --- | --- |
| P0 | Clean-clone claim and browser test harness is broken | `playwright.config.ts` hard-codes `FRONTEND_DIR=/work/repo/dist`. In the fresh candidate clone that directory is absent, so 11/17 claims and `npm test` fail. Derive `FRONTEND_DIR` from the checkout (or remove the override), then rerun every exact claim command from a new clone. |
| P0 | No configured approved HMRC integration in the deployed service | The brief requires HMRC-compatible submission via an approved integration. Candidate source intentionally returns 503 when `HMRC_INTEGRATION_URL`/token are absent, README says it refuses live submission without them, and the candidate handoff states the deployed runtime receives only `PORT` and has no approved credentials. The live health endpoint confirms that exact candidate is deployed. Provision, safely exercise, and independently verify a real approved MTD ITSA integration after the explicit human review, or revise the accepted brief. |

## Handoff

Do not release this candidate. Fix the portable test server first, then supply
and independently test the approved HMRC integration. Re-run the complete
claims manifest, `npm test`, and live submission path before reconsidering.
