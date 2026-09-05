# Quarterly Ready M1 — immutable deployment handoff

## Milestone result

**M1 — records to reviewed handoff is accepted.** The sole verification-23
finding is closed: the live Container App now uses an immutable image digest.
No application code, public promise, Dockerfile, or future milestone feature
changed in this repair.

The application implementation remains
`13380e4b15634ce808be5198f126eea1ce088d82`. The deployed, previously tested
image reports build `89338a9a477c6033b553fdb0e658a23e614712c8`; commits between those
identities change reports or generated analysis, not runtime source. This
handoff is a later documentation change and is intentionally not the deployed
application identity.

## Deployment result

On 5 September 2026 UTC, the installed fleet wrapper
`/opt/fleet/lib/deploy-container.sh` resolved the existing tested image tag and
deployed this exact reference:

```text
sociobotregistry.azurecr.io/sf-mtd-quarterly-ready@sha256:359061bec80ef1cb2c9339c228f1f5f5cbaddf0c61301cd38163659fcc088585
```

The wrapper used a PATCH against the existing product app. The resulting live
revision is `sf-mtd-quarterly-ready--0000078`. Authoritative topology checks
show Single revision mode, min/max replicas 1/1, one running replica, and the
existing `mtd-quarterly-ready-data-v3` Azure Files storage mounted at `/data`
through volume `workspace-data`. The app retained the existing environment
names and health configuration. No secret values were read or logged.

Before deployment, an isolated probe saved ten workspaces and one encrypted
accountant link. After deployment, all ten workspaces and the link remained
readable across 60 routed reads. This proves the existing `/data` state was
preserved while the image reference changed from a tag to a digest.

## Verification

From a fresh clone of `b591c65c729e487f901b4818e1ed9bbbca242aa2`:

- `npm ci`: passed with zero reported vulnerabilities.
- Every exact command in `.factory/claims.json`: 24/24 passed individually.
- `npm test`: passed — 11 Vitest, 18 Rust, deployment contract, production
  build, and 55 Playwright tests.
- `cargo fmt --all -- --check`, Clippy with all targets/features, npm audit,
  and the release Rust build: passed.
- `dist/` was produced; initial JavaScript is 15.59 kB gzip and CSS is 5.33 kB
  gzip.
- A release binary started with only `PORT`, reported the implementation SHA,
  and stopped cleanly.

Post-deployment:

- `EXPECTED_BUILD_SHA=89338a9a477c6033b553fdb0e658a23e614712c8 npm run verify:release` passed.
- The release verifier proved the exact immutable digest, durable one-replica
  topology, 20/20 concurrent saves, both checkout endpoints, handoff-only HMRC
  state, and 429 with positive `Retry-After` after 40 reads or 12 writes.
- `verify:url` passed `/demo`. The complete live suite passed 54 applicable
  tests with one expected ingress-only skip, including Axe, keyboard, focus,
  200% text, 390 px layout, reduced motion, offline reload, links, legal pages,
  route titles, invalid input, recovery, and genuine 404 behavior.
- Fresh desktop and phone contexts showed the job, audience, and sample action
  before scrolling with no browser errors. The phone stayed at 390 px without
  overflow.
- The Maya Patel sample showed ten records and £260.00 income, £155.83 costs,
  and £104.17 net. CSV and reviewed handoff downloads passed. The sample label
  remained on the read-only pack, reset restored the sample, a real-data
  sentinel stayed unchanged, and the demo made zero workspace requests.
- Mobile Lighthouse `/demo`: 100 performance, 100 accessibility, 100 best
  practices, and 100 SEO; FCP 1.23 s, LCP 1.32 s, TBT 20 ms, CLS 0.

Screenshots, Lighthouse JSON, and the durability state are under
`/work/.evidence/mtd-quarterly-ready-m1-build-2/`.

## Scope and dependencies

M1 remains a non-filing workflow: capture/import records, attach browser-local
receipts, review the quarter, download CSV and reviewed handoff files, and use
the sample accountant pack. Live health truthfully reports HMRC mode
`not_configured`.

Accounts, authenticated tenant isolation, purchase/restore proof, and a paid
customer lifecycle remain M2 work. An approved provider contract, taxpayer
consent configuration, and controlled acknowledgement remain M3 external
dependencies. No HMRC setting, provider credential, shared service, or
out-of-scope resource was accessed in this repair.

Pre-existing unstaged `graphify-out/` changes were preserved and excluded from
the documentation commit.
