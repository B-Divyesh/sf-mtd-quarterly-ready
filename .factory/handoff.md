# Quarterly Ready — verification 22 handoff

## Status: FAIL

Independent verification found **2 findings and 0 untested claims**. The scoped
isolation cleanup passes, but the product cannot complete the brief's required
approved HMRC submission. The styled 404 also uses metaphorical copy prohibited
by the plain-words contract.

## Revisions

- Isolation/deployment candidate: `3e34f53e0d4b5ce13d058a3d2dd6f238388c088c`.
- Documentation checkout: `1c893635f66aeb48eec3270dee7cdf25993ceb52`.
- Live runtime build: `7c840e4853bbcb16270977bdb568271ebd86c746`.
- Last application-code change in that runtime lineage:
  `a428876efd57704a2617e67e983ffe561b6abee0`.

The live app assets match the clean build byte for byte. The later isolation
and Graphify changes do not alter the compiled application.

## What passed

- All 24 exact claim commands passed from a fresh clone after `npm ci`.
- `npm test` passed: typecheck, 11 Vitest, 18 Rust, deploy contract, production
  build, and 54 Playwright tests.
- Formatting, Clippy with warnings denied, npm audit, and release build passed.
- The live suite passed 53 tests with one inapplicable non-claim fallback skip.
- Desktop and 390 px phone first screens, one-click demo, realistic output,
  persistent demo banner, reset, and real-data isolation passed.
- Accessibility, keyboard, focus, 200% text, reduced motion, legal routes,
  offline reload/update, link crawl, security headers, and genuine 404 status
  passed.
- Lighthouse `/demo`: 100 performance, 100 accessibility, 100 best practices,
  100 SEO; LCP 1.29 s and CLS 0.
- Live release verification passed one-replica `/data` topology, immutable
  image identity, concurrent persistence, both checkout routes, and exact
  40-read/12-write limits with 429 plus `Retry-After`.
- A local two-start exercise restored ten workspaces and an accountant link
  from encrypted durable storage.
- Isolation checks passed shell syntax, fail-closed mode rejection, deployment
  contract, and executable-config inspection. No shared-vault or HMRC secret
  reference remains.

## Open findings

1. Critical: live HMRC submission through an approved integration remains
   unavailable. The handoff-only deployment is honest and isolated, but it
   does not complete the researched job.
2. Minor: the 404 uses `NO SIGNAL` and `This page is not on the panel`; replace
   that metaphorical copy with direct page-not-found wording.

## Reproduce

```sh
npm ci
npm test
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
npm audit --audit-level=high
npm run test:deploy-contract
EXPECTED_BUILD_SHA=7c840e4853bbcb16270977bdb568271ebd86c746 npm run verify:release
VERIFY_ORIGIN=https://mtd-quarterly-ready.sociobot.in \
  EXPECTED_BUILD_SHA=7c840e4853bbcb16270977bdb568271ebd86c746 \
  npx playwright test
```

Full evidence and prior-finding disposition are in
[`verification-22.md`](verification-22.md). No product code, deployment, or
infrastructure was changed. Pre-existing `graphify-out/` changes remain
unstaged.
