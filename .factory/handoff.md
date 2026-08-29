# Quarterly Ready — repair 11 handoff

## Release status

**BLOCKED — not a releasable submission service until an approved HMRC
integration is provisioned.**

The deployed repair artifact is commit
`a84fc7d71e40e0f26c96163bb51125b48e77d728`, image
`sociobotregistry.azurecr.io/sf-mtd-quarterly-ready:a84fc7d`
(`sha256:870d0b113646d8d1ea5add8928ee8c0c71bb08e00891788474bac58a54f5649d`),
on live revision `sf-mtd-quarterly-ready--0000032`.

This repair resolves the durable-state, replica, rate-limit, claim-registry,
and deployment-contract findings from independent verification 11. It does not
pretend that the missing HMRC credentials are fixed: live `/health` reports
`hmrc_integration_configured:false`, and the hardened release command correctly
fails on that condition.

## What changed

- Reconfigured the live Container App to one running replica with the registered
  Azure Files `mtd-quarterly-ready-data-v3` volume mounted at `/data`. This
  restores the SQLite snapshot, encryption key, audit log, real workspaces, and
  accountant links to durable storage and returns the external rate allowance to
  40 reads / 12 writes per client per second.
- Proved a synthetic workspace survives both a replica restart and a revision
  replacement using `scripts/verify-durability.mjs`.
- Added three exact registered claims and regressions:
  `receipt-locality` records the outgoing workspace body and IndexedDB state;
  `quarter-record-separation` checks distinct browser keys, workspace IDs, and
  restored server documents; `conditional-submission` checks the control for
  configured and unavailable server capability.
- Added `npm run verify:release`. It requires both the Azure one-replica/
  Azure-Files topology and `hmrc_integration_configured:true`; it cannot approve
  a handoff-only deployment as the researched submission product.
- The deployment script now supplies `Content-Type: application/json` to its
  Azure PATCH and checks the two managed Key Vault secret references before any
  ACR build or Container App mutation. The contract test protects both details.

## Verified

Local clean-install and quality evidence:

```text
npm ci                                                       pass (60 packages, 0 vulnerabilities)
npm run typecheck                                            pass
npm run test:unit                                            pass (11 tests)
cargo test                                                   pass (13 tests)
cargo fmt -- --check                                         pass
cargo clippy --all-targets -- -D warnings                    pass
npm run test:deploy-contract                                 pass
npm test                                                     pass (42 Chromium, 13 Rust, 11 Vitest)
npm run build                                                pass (dist/; 44.69 KB JS, 21.67 KB CSS)
BUILD_SHA=a84fc7d... cargo build --release                  pass
bash -n scripts/deploy-container.sh                          pass
node --check scripts/verify-live.mjs                         pass
```

Each of the 21 `.factory/claims.json` commands was run independently after the
clean install; all passed. The full live demo/claim suite also passed: 15
Playwright scenarios cover every registered browser claim, CSV, receipt quota
recovery, privacy request logging, offline reload, and checkout behavior.

Production evidence at `https://mtd-quarterly-ready.sociobot.in`:

```text
EXPECTED_BUILD_SHA=a84fc7d... npm run verify:live            pass
npm run verify:topology                                      pass
durability seed → replica restart → check                    pass
durability check after revision 0000031 → 0000032            pass
VERIFY_ORIGIN=https://mtd-quarterly-ready.sociobot.in \
  npx playwright test tests/claims.spec.ts                   pass (15)
VERIFY_ORIGIN=https://mtd-quarterly-ready.sociobot.in \
  npx playwright test tests/accessibility.spec.ts --grep ... pass (12)
```

Live browser checks covered desktop, 390 px mobile layout and 44 px targets,
keyboard demo entry, route metadata, empty real-records load, internal-link
crawl, service-worker offline reload/update, demo isolation, and no
third-party/cookie traffic. The Playwright Axe integration found zero serious
or critical issues on `/`, `/demo`, `/privacy`, and `/terms`. The factory
`verify-url.sh` passed live `/` and `/demo` with correct title, `lang`, one H1,
main landmark, image alt text, and zero console errors. The standalone
`@axe-core/cli` was also attempted with the preinstalled Chromium path but its
Selenium/ChromeDriver launcher exited before auditing; the Playwright Axe
integration is the authoritative successful accessibility evidence.

The ACR multi-stage container build passed in Azure (Docker is not installed in
this worker). The image uses the existing non-root runtime and starts with only
its required `PORT` configuration; live health reported the exact build SHA and
`safe_qa_fixtures:true`. Live headers include CSP with `frame-ancestors 'none'`,
HSTS, `nosniff`, strict-origin referrer policy, permissions policy, and
`no-cache` HTML policy.

## Remaining blocker and next step

The Key Vault metadata lookup for both required secrets failed with
`SecretNotFound`:

- `mtd-quarterly-ready-hmrc-integration-url`
- `mtd-quarterly-ready-hmrc-integration-token`

No real approved HMRC endpoint or token is present in this repository or the
available Key Vault. Creating a fake endpoint or test token would make a false
submission claim, so it was not done. Provision both secrets as managed Key
Vault references for the existing user-assigned identity, then run:

```sh
bash scripts/deploy-container.sh
EXPECTED_BUILD_SHA=<deployed-commit> npm run verify:release
```

The latter must complete a provider-approved sandbox submission after the
human-review gate before this product can be marked releasable. Until then, the
live UI correctly exposes the reviewed HMRC handoff instead of a direct
submission control.
