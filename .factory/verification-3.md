# Independent verification 3 — FAIL

Verified 2026-08-28 against candidate commit
`fb8d5f29b93709dfd508a0220cd752e151504088` and
`https://mtd-quarterly-ready.sociobot.in`.

## Decision

**FAIL.** The deployed candidate is the requested commit and most technical
checks pass, but it does not deliver the brief's required HMRC-compatible
submission through an approved integration. It also emits a browser console
error on the normal empty-records path, violating the no-console-errors quality
gate.

No product code was changed in this verification.

## First read and deployment identity

A cold desktop load of `/` returned 200. The first screen plainly says it
turns records into a checked quarterly update, names UK sole traders, tutors
and landlords, and puts **Try it with sample data** beside the explanation
“Opens a private sample quarter. No account needed.” The one-click link opens
`/demo` and its persistent banner says “Demo — sample data, nothing is saved.”
This mandatory first-read/demo gate passes.

Live identity and artifact comparison both match the candidate:

- `GET /health` returned
  `{"status":"ok","build_sha":"fb8d5f29b93709dfd508a0220cd752e151504088"}`.
- SHA-256 of local `dist/assets/index-DScAhmUU.js` and
  `index-DBPfy_jM.css` exactly matched the corresponding live assets.

## Mandatory claims gate — PASS

`.factory/claims.json` exists and contains 17 claims. After `npm ci` from this
checkout, I ran every exact listed command individually against the configured
demo entry point. All passed:

| Claim coverage | Command type | Result |
| --- | --- | --- |
| demo isolation/access, no tracking, CSV export, quarter review, CSV import, receipt capture, handoff, demo accountant link, licence gate, offline copy, paid tier | 12 exact `npx playwright test --grep @claim:…` commands | pass |
| approved-integration submission fixture, 30-day link expiry, encrypted storage, audit chain, anonymous page count | 5 exact `cargo test claim_…` commands | pass |

The clean-run Playwright status file is `{"status":"passed","failedTests":[]}`.
The claim tests prove the declared demo behaviour; the approved-integration
claim uses a local mocked bridge and therefore does not establish a configured
live integration.

## Local build and automated checks — PASS

- `npm ci`: passed; 0 reported package vulnerabilities.
- `npm test`: passed: typecheck, 4 Vitest tests, 8 Rust tests, production Vite
  build, and 24 Playwright tests.
- `npm run build`: passed and produced `dist/`.
- `cargo clippy -- -D warnings`: passed.
- `BUILD_SHA=fb8d… cargo build --release`: passed. The release binary started
  with only `PORT=8180`; `/health` returned the candidate SHA.
- Initial built JavaScript is 35.37 KB / 11.66 KB gzip; CSS is 21.07 KB /
  5.27 KB gzip, within the applicable budgets. The largest shipped hero image
  is 51.7 KB.
- Docker could not be executed because this verifier environment has no
  `docker` command. The deployed build and local release-binary checks above
  provide deployment evidence, but not a fresh local container-build result.

## Product, accessibility, privacy, PWA, and backend exercise

- Live Axe found zero serious/critical violations on `/`, `/demo`, `/records`,
  `/privacy`, `/terms`, and the not-found route. Each had one H1, one main
  landmark, `lang=en-GB`, and the correct route title.
- At 390×844, `/demo` had `scrollWidth == clientWidth == 390`. At simulated
  200% text, it still had no horizontal overflow and its primary button was
  358×45 px. Keyboard focus showed a designed 3 px teal outline; the skip
  link was reachable.
- Demo privacy: a fresh live `/demo` context made requests only to
  `https://mtd-quarterly-ready.sociobot.in`, set no cookies, and stored only
  `demo:quarterly-ready:document`. Changing the sample category did not call
  a workspace API.
- Security headers on HTML and assets included CSP with `frame-ancestors
  'none'`, `X-Content-Type-Options: nosniff`, Referrer-Policy, and
  Permissions-Policy. Hashed JS/CSS use `Cache-Control: public,
  max-age=31536000, immutable`.
- The live service worker controls `/demo`; `registration.update()` completed,
  cache `quarterly-ready-v2` was active, and an offline reload showed both
  “Offline — browser copy active” and the Maya Patel sample.
- Normal real-record exercise passed: empty state, invalid £0 amount recovery,
  quarter-date boundary rejection, malformed-CSV recovery, valid CSV import,
  encrypted server save, reload persistence, and a confirmed delete path.
  A 100-request concurrent live `/health` smoke returned 100×200.
- Live API rate limiting is enforced. One client received 40 ordinary read
  responses and 12 read `429`s under 52 concurrent calls; writes returned
  12×204 then 429. `429` responses carried `Retry-After: 1`. Observed
  allowance: 40 reads/s and 12 writes/s; `/health` is exempt.

## Release-blocking defects

| Severity | Finding | Fresh evidence and required resolution |
| --- | --- | --- |
| P0 | No live approved HMRC integration | The brief's smallest useful product requires “HMRC-compatible submission via an approved integration.” The factory runtime contract deploys with only `PORT`. The independently built release binary under that configuration logged `hmrc_integration:"not_configured"`; source defaults have no integration, and the existing handoff confirms no approved credentials exist. The app therefore refuses the paid submission after licence verification. Provision and independently exercise a real approved MTD ITSA integration with explicit human review, or revise the acceptance contract. |
| P1 | Console error on normal `/records` first load | A fresh live `/records` visit gets the expected `GET /api/workspace` 404 for an empty workspace and renders the correct empty state, but Chromium logs `Failed to load resource: the server responded with a status of 404 ()` for that request. `/`, `/demo`, `/privacy`, `/terms`, and the 404 page had no errors. Handle the expected empty state without a console error (for example, return a non-error empty-workspace response) and add a regression test. |

## Handoff

The candidate is not releasable until both P0 and P1 are resolved and the
claims/production checks are rerun. The previous subscription-link bypass is
fixed: the unauthenticated live-style share claim now returns 402, and monthly
and annual Sociobot subscription checkout links are present.
