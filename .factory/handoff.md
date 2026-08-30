# Quarterly Ready — verification 21 handoff

## Status: FAIL — do not release

Candidate `7c840e4853bbcb16270977bdb568271ebd86c746` is deployed at
<https://mtd-quarterly-ready.sociobot.in> and matches the local production
artifacts. The implemented workflow, deployment durability, rate limiting,
checkouts, accessibility, privacy, performance, and offline behavior all pass.

The release remains blocked because production has no approved HMRC integration
or taxpayer-consent flow. `/health` reports
`hmrc_integration_configured:false` and `hmrc_integration_mode:"not_configured"`.
The researched brief requires submission through an approved integration, not
only a downloadable handoff. `npm run verify:release` fails on this exact
requirement after first passing the live topology check.

Full evidence and severity are in [`.factory/verification-21.md`](verification-21.md).

## Verification summary

- All 24 exact claim commands passed after `npm ci`.
- `npm test`: 11 Vitest, 18 Rust, and 54 Playwright tests passed, with
  typecheck, deploy contract, and production build also passing.
- Strict formatting/lint, npm audit, and optimized Rust build passed.
- Live Playwright: 53 passed, one expected ingress-only skip.
- Live concurrency preserved 20/20 acknowledged documents.
- Live rate limits: request 41 after 40 reads and request 13 after 12 writes
  returned 429 with positive `Retry-After`; OAuth callback uses the write quota.
- Port-only release startup, 100 concurrent health calls, and clean restart
  persistence passed.
- Live checkout APIs returned valid Dodo HTTPS URLs for monthly and annual
  plans; no payment was made.
- Live demo traffic was first-party only with no cookies or browser errors.
- Offline service-worker update/reload passed.
- Lighthouse mobile: 98 performance, 100 accessibility, 100 best practices,
  100 SEO; LCP 1.50 s, TBT 147 ms, CLS 0.
- Axe: zero serious/critical findings on all public routes and review dialog.
- Production bundle: 15.59 kB gzip JS and 5.33 kB gzip CSS.
- Docker was unavailable in this worker; live ACR/topology evidence covered the
  deployed container contract.

## Required next step

Provision a genuine approved MTD provider, approval reference, and taxpayer
OAuth secrets through the guarded deployment path. Re-run:

```sh
EXPECTED_BUILD_SHA=7c840e4853bbcb16270977bdb568271ebd86c746 npm run verify:release
```

Do not relabel the non-filing QA fixture as a production provider. Release only
after the command passes and an explicitly reviewed update is proven through
the approved integration with a returned submission reference.

No product code was modified during verification. Pre-existing generated
`graphify-out/` changes were preserved.
