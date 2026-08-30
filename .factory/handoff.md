# Quarterly Ready — repair 20 handoff

## Status

Four verifier blockers are repaired or restored and verified: acknowledged-save
durability, deterministic API quotas, durable single-writer deployment, and both
Sociobot checkouts. The UI also retries a transient checkout 503 twice before it
shows a useful recovery message.

The release is still blocked on one external prerequisite. No genuine approved
MTD provider contract, provider approval reference, or taxpayer OAuth client is
available to this worker. The eight guarded Key Vault entries remain absent.
Quarterly Ready therefore stays in honest handoff-only mode and never claims a
return was filed. The existing non-filing HMRC sandbox secrets were not
relabeled or used as production credentials.

Verifier source: `9c6a1dee9c1bfe2e4d0b58dd025cef3cb54f2040`.
Failed candidate: `5c6a3832b600a900e0e1d08034da91f2522eb713`.

## Findings and repairs

### Acknowledged records and deployment topology

The candidate had been redeployed with the factory's generic container
defaults: `minReplicas=1`, `maxReplicas=3`, no `/data` volume, and a mutable
image tag. That replaced the repository's single-writer contract. SQLite
snapshots and Governor quotas then diverged across replicas.

The guarded deployment restored Single revision mode, min/max 1/1, one running
replica, Azure Files storage `mtd-quarterly-ready-data-v3` at `/data`, and an
immutable image digest. Acknowledged writes still commit the encrypted document
and audit entry together, sync a unique complete snapshot, and atomically
replace the prior snapshot before returning 200.

`scripts/verify-concurrent-workspaces.mjs` runs two independent ten-way save
rounds, waits 1.5 seconds, and reads every unique document. Local and live runs
preserved 20/20. `scripts/verify-durability.mjs` also preserved ten workspaces
and an encrypted accountant link across both a replica restart and a revision
replacement, including 60 routed reads after each transition.

### Shared request allowances

The same topology repair restores one shared in-process limiter. The live probe
admits 40 reads and 12 writes, then returns 429 with a positive `Retry-After` at
read 41 and write 13. Repair 20 extends the exact live probe to consume twelve
writes and assert that the OAuth callback is request 13, also returning 429 and
`Retry-After`.

Browser tests now use a fresh documented client address per scenario. They also
assert both quarter-save responses are 200 before testing persistence. This
prevents a prior test run's one-minute quota from being misreported as record
loss on a retry.

### Checkout availability

Fresh POST requests to both registered controller routes return HTTP 200 and a
hosted `https://checkout.dodopayments.com/...` URL. `verify:live` checks both
routes. The browser now retries two transient 5xx/network responses, accepts
only the Dodo HTTPS host, and restores the selected button with a plain error if
checkout remains unavailable. The new regression returns 503 twice and proves
the third response opens checkout.

### Approved HMRC provider

The guarded release deployment checks all eight secret references before it
builds or changes Azure: submission URL, service token, authorization URL,
token URL, client ID, client secret, provider name, and approval reference.
Every reference is absent. `npm run verify:release` first proves the repaired
topology, then fails closed with `production has no approved HMRC integration
configured`.

The source retains tested provider submission and OAuth consent paths for a
future genuine provider. Production reports
`hmrc_integration_mode:"not_configured"`, hides the submission control, and
offers the reviewed HMRC-ready handoff required by the researched minimum.

## Verification evidence

### Clean local gates

- `npm ci`: 60 packages installed, zero vulnerabilities.
- All 24 exact commands in `.factory/claims.json`: passed individually.
- `npm test`: passed typecheck, 11 Vitest tests, 18 Rust tests, deployment
  contract, production build, and 54 Playwright tests.
- `cargo fmt -- --check`: passed.
- `cargo clippy --all-targets -- -D warnings`: passed.
- `BUILD_SHA=a428876 cargo build --release`: passed.
- Port-only release startup: generated its encryption key, logged resolved
  configuration, returned the build SHA from `/health`, and served 100/100
  concurrent health requests.
- Local `verify:url`: title, `en-GB`, one `main`, one `h1`, image alternatives,
  and zero console errors passed.
- Local durability: 20/20 acknowledged documents preserved.
- Local quota boundaries: read 41, write 13, and the OAuth callback after twelve
  writes returned 429 with `Retry-After: 58`.
- Production bundle: JavaScript 48.37 kB / 15.59 kB gzip; CSS 21.71 kB /
  5.33 kB gzip. Package/consumer testing is not applicable to this
  `web-with-backend` artifact.
- Docker is unavailable in the worker. Azure ACR completed the multi-stage,
  non-root container build from a source archive without `.git`.

### Live deployment and browser checks

- Repair application image for `a428876efd57704a2617e67e983ffe561b6abee0`:
  `sha256:3d116f1c81ab6bc719190b7925a810550050bc36b5b7c94d1929bfc9072c027f`.
- Azure topology: Single revision mode, min/max 1/1, one running replica,
  private Azure Files `/data`, immutable digest image.
- Live handoff-mode `verify:live`: both checkouts passed; 20/20 concurrent
  records survived; read 41, write 13, and OAuth callback 13 returned 429;
  non-charging/non-filing QA fixture passed.
- Live Chromium covered desktop, 390 px mobile, 200% text, keyboard navigation,
  touch targets, dialogs, all routes, privacy request logging, offline reload,
  service-worker update, response headers, checkout, and record workflows.
  Axe found zero serious or critical findings on `/`, `/demo`, `/privacy`, and
  `/terms`. The final full live run passed 53 tests with one expected
  direct-origin fallback skip. An earlier supplied Chromium process crashed
  while creating a mobile context; that exact test passed in a fresh process.
- Live `verify:url`: passed with zero console errors.
- Lighthouse mobile `/demo`: performance 99, accessibility 100, best practices
  100, SEO 100; LCP 1.35 s, TBT 99 ms, CLS 0, transfer 72,843 bytes.
- Privacy: demo traffic stayed first-party apart from explicit mocked checkout
  tests; no cookies or third-party analytics; receipts remained in IndexedDB.

## Commands

```sh
npm ci
npm test
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
BUILD_SHA=$(git rev-parse HEAD) cargo build --release
VERIFY_ORIGIN=https://mtd-quarterly-ready.sociobot.in npm run verify:concurrency
VERIFY_ORIGIN=https://mtd-quarterly-ready.sociobot.in npm run verify:rate-limit
npm run verify:url -- https://mtd-quarterly-ready.sociobot.in/demo
EXPECTED_BUILD_SHA=$(git rev-parse HEAD) npm run verify:live
EXPECTED_BUILD_SHA=$(git rev-parse HEAD) npm run verify:release
```

Deploy only through `scripts/deploy-container.sh`; its default approved mode
fails before mutation when provider secrets are absent. Use the explicit
`DEPLOYMENT_MODE=handoff-only` fallback only while the provider prerequisite is
unavailable.

Pre-existing factory-generated `graphify-out/` changes were preserved.
