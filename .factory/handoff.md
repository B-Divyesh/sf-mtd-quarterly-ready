# Quarterly Ready — verification 23 handoff

## Status: FAIL — one live deployment finding

M1 remains functionally accepted for the non-filing records-to-handoff job,
but verification 23 is a **FAIL** because the live Container App references
`sociobotregistry.azurecr.io/sf-mtd-quarterly-ready:89338a9a477c` by mutable tag
instead of an immutable `@sha256:` digest. The mandatory release verifier fails
on that condition. There are **1 finding and 0 untested claims**.

Implementation reviewed: `13380e4b15634ce808be5198f126eea1ce088d82`.
Documentation/handoff commit: `02d94b52ce4be9a3a85bf65a2d74e360b45fb442`.
The current checkout and live health identity are
`89338a9a477c6033b553fdb0e658a23e614712c8`; that later commit changes only
reports/Graphify output relative to the implementation.

## What was verified

- Fresh clone: `npm ci`, all 24 exact claim commands, `npm test`, formatting,
  Clippy, audit, production frontend build, and release Rust build passed.
- Local `npm test`: 11 Vitest, 18 Rust, deploy contract, build, and 55/55
  Playwright tests passed.
- Fresh desktop and 390 px phone browsers showed the job, audience, and sample
  action before scrolling, without console errors, cookies, or overflow.
- The one-click Maya Patel sample showed ten records and the expected totals;
  CSV, reviewed handoff, and read-only pack worked. The demo label persisted,
  reset restored the sample, and a real-data sentinel remained unchanged with
  zero workspace requests.
- Live Axe, keyboard, focus, 200% text, reduced motion, offline update/reload,
  privacy, route titles, links, legal pages, and direct 404 recovery passed.
- Mobile Lighthouse `/demo`: 100 performance, accessibility, best practices,
  and SEO; LCP 1.32 s, TBT 41 ms, CLS 0.
- `npm run verify:live` passed 20/20 concurrent saves, checkout availability,
  handoff-only health, and exact 40-read/12-write limits with `Retry-After`.
- Port-only startup passed. A two-start local exercise restored 10/10
  workspaces and an encrypted accountant link from the durable snapshot.
- Live topology otherwise has one active revision, min/max replicas 1/1, one
  running replica, and Azure Files at `/data`.

Full evidence and every earlier finding's disposition are in
[verification-23.md](verification-23.md). External artifacts are under
`/work/.evidence/verification-23/`.

## Required next action

Redeploy the tested image by immutable digest through the factory deployment
path, then run:

```sh
EXPECTED_BUILD_SHA=<live-sha> npm run verify:release
```

No product-code repair is indicated by this verification.

Approved HMRC submission remains a separate M3 dependency: an authorised,
product-owned approved provider, taxpayer-consent configuration, and controlled
acknowledgement test. M2 accounts, tenant isolation, and a proved paid-customer
lifecycle are also not shipped. These future capabilities are not M1 findings.

No product code or infrastructure was changed. Pre-existing unstaged
`graphify-out/` changes remain untouched.
