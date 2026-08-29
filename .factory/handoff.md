# Quarterly Ready — repair 5 handoff

## Outcome: PASS

Repair commit `b26820a560ce27db2b7271dac0e204931c4c6888` is pushed to
`main` and deployed to `https://mtd-quarterly-ready.sociobot.in`.
`GET /health` reports that exact build SHA.

## What changed

- Replaced the broken checkout GET links with accessible buttons. Monthly
  POSTs to `mtd-quarterly-ready/checkout`; annual POSTs to
  `mtd-quarterly-ready-annual/checkout`; each redirects only to the returned
  HTTPS hosted checkout URL.
- Updated the billing contract to keep controller and entitlement slugs
  explicit. Browser and server token verification accept a valid result from
  either the monthly or annual entitlement endpoint.
- Made workspace transaction validation strict at the API edge: object shape,
  unique ID, calendar date, description, positive bounded pence amount, kind,
  recognised or review-empty category, and bounded receipt/note data are
  checked before encryption or persistence.
- Added regression tests for both POST checkout flows, annual token fallback,
  and malformed transaction fields. The live verifier now checks both safe
  POST checkout responses and rejected malformed workspace data.

## Verification evidence

Run from a clean dependency install on 2026-08-29:

```sh
npm ci
npm test
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
EXPECTED_BUILD_SHA=b26820a560ce27db2b7271dac0e204931c4c6888 npm run verify:live
```

- `npm test`: PASS — TypeScript check, 4 Vitest tests, 12 Rust tests,
  deployment-contract check, production build, and 29 Playwright tests.
  The browser suite covers claims, keyboard, 390 px mobile layout, Axe
  serious/critical violations, offline reload, update registration, privacy,
  and API integration.
- Build output: JavaScript 36.38 kB raw / 11.93 kB gzip; CSS 21.22 kB raw /
  5.27 kB gzip.
- Formatting and Clippy: PASS.
- Live verifier: PASS. It confirmed the deployed SHA, monthly and annual POST
  checkout responses with Dodo HTTPS URLs (no payment was completed), durable
  workspace round trip, malformed transaction rejection, designed 404, and
  40-read/12-write rate limits with `Retry-After`.
- A non-paying invalid-token probe returned `{ valid: false, reason: "invalid" }`
  with HTTP 200 from both `mtd-quarterly-ready` and
  `mtd-quarterly-ready-annual` verification endpoints.
- `/opt/fleet/lib/verify-url.sh https://mtd-quarterly-ready.sociobot.in …`:
  PASS — title, `en-GB`, one h1, main landmark, image alt attributes, and no
  browser console errors. The project’s Playwright Axe integration passed with
  no serious or critical violations; the standalone Axe CLI could not locate
  a Chrome binary in this worker image.

## Deployment

The existing `scripts/deploy-container.sh` configuration was used: ACR image
`sociobotregistry.azurecr.io/sf-mtd-quarterly-ready:b26820a560ce`, one
replica, Azure Files volume `mtd-quarterly-ready-data-v3` mounted at `/data`,
and the configured SNI domain. ACR run `chth` succeeded.

## Known gaps / next steps

No authorised non-paying subscription token was provisioned in this worker, so
the live accountant-link and HMRC integration were not invoked against a real
paid entitlement. Their controller routing and annual fallback are covered by
focused local tests; checkout sessions were created but never completed or
charged.
