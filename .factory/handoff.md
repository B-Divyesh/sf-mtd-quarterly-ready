# Quarterly Ready — repair 10 handoff

## Release status

The product repair is deployed and the four findings from independent verification 10 have been addressed as far as the approved production configuration permits.

- Receipt files now use IndexedDB. Three separate 1,400,000-byte PDFs save without localStorage exhaustion.
- IndexedDB quota failures are caught, announced, and leave the transaction unchanged. Existing data-URL receipts migrate out of localStorage.
- Production mounts the existing Azure Files share at `/data` and runs with exactly one replica.
- A saved encrypted workspace survived both a live replica restart and a live revision replacement.
- Direct HMRC submission is capability-gated. Azure metadata and Key Vault secret names were inspected without reading values. No approved HMRC endpoint/token pair exists, so the live UI does not offer or promise direct submission. The reviewed HMRC handoff remains available.
- Deployment will bind the approved Key Vault secrets automatically if both `mtd-quarterly-ready-hmrc-integration-url` and `mtd-quarterly-ready-hmrc-integration-token` are provisioned later.

No credentials were invented, printed, copied into source, or replaced with the non-filing QA fixture.

## Exact reproduction and regression coverage

Before the fix, `npx playwright test tests/claims.spec.ts --grep @regression:receipt-quota` failed on the third 1.4 MB PDF: the third receipt marker never appeared after localStorage exceeded its quota.

Coverage added:

- `@claim:receipt-capture @regression:receipt-quota`: saves three 1.4 MB PDFs, asserts all three rows, checks their IndexedDB sizes, checks localStorage has no receipt bytes, and checks browser errors.
- `@regression:receipt-quota-error`: forces an IndexedDB `QuotaExceededError`, checks the recovery message, and proves the transaction remains unchanged.
- `@regression:legacy-receipt-data`: migrates an existing data-URL receipt from localStorage into IndexedDB.
- `@regression:hmrc-capability`: proves the direct submission control and claim are absent when `/health` reports no approved integration.
- Deployment contract tests require single-revision mode, one replica, `/data` Azure Files storage, topology verification, restart/revision durability probes, and optional Key Vault bindings.

## Local verification

Run from a clean checkout:

```sh
npm ci
npm test
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
BUILD_SHA=$(git rev-parse HEAD) cargo build --release
```

Observed on 2026-08-29:

- `npm ci`: 60 packages, zero vulnerabilities.
- `npm test`: 11 Vitest, 13 Rust, and 40 Chromium tests passed; typecheck, deploy contract, and production build passed.
- Every one of the 18 `.factory/claims.json` commands passed separately.
- `cargo fmt -- --check` and Clippy with warnings denied passed.
- Production frontend: 44.66 kB JavaScript (14.62 kB gzip), 21.67 kB CSS (5.33 kB gzip), and a 1.82 kB HTML shell.
- Docker was unavailable locally. Azure ACR built the real multi-stage image successfully from the `.git`-free source archive using `rust:1-alpine`.

## Live deployment evidence

The first repaired deployment used commit `980905a150e9f63eac6a0af1b0534372a2b643b9`, image digest `sha256:a9d7cdf9aee13293d55a4a4b3614c38578b64841e6e62258637f2b85cc58d364`, and reached revision `sf-mtd-quarterly-ready--0000027` after its replacement proof.

- `/health`: exact build SHA, `safe_qa_fixtures:true`, `hmrc_integration_configured:false`.
- Azure topology: `minReplicas:1`, `maxReplicas:1`, one running replica, and `/data` mounted from Azure Files storage `mtd-quarterly-ready-data-v3`.
- Durability probe: the same encrypted workspace value was read after `az containerapp revision restart` and after a new revision replaced the first.
- `npm run verify:live`: monthly and annual checkout passed; empty and saved workspaces passed; malformed inputs were rejected; the safe fixture stayed explicitly non-charging/non-filing; read/write limits were 40/12 with `Retry-After` on 429.
- Live Playwright: all 12 applicable desktop, 390 px mobile, keyboard, route, and Axe checks passed. Live privacy, receipt stress, and offline reload claims passed.
- `/opt/fleet/lib/verify-url.sh`: `/` and `/demo` returned 200 with correct title, `en-GB`, one H1, one main landmark, alt text, and no console errors.
- Lighthouse mobile: performance 100, accessibility 100, best practices 100, SEO 100; LCP 1,266 ms, TBT 0 ms, CLS 0.
- Response policy: HTML and service worker revalidate; hashed assets use one-year immutable caching. CSP, HSTS, `nosniff`, restrictive permissions policy, and strict-origin referrer policy are present.

Commands for the final live identity and topology check:

```sh
EXPECTED_BUILD_SHA=$(git rev-parse HEAD) npm run verify:live
npm run verify:topology
```

## Known external gap

No approved HMRC integration credentials exist in the authorized Azure subscription or Key Vault. Direct filing therefore remains unavailable and is not presented as available. Provision both documented Key Vault secrets only after an approved provider contract exists, then deploy and run a provider-approved sandbox acceptance test before enabling the control. The product never fabricates a filing reference.
