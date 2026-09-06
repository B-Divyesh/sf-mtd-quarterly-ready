# Quarterly Ready — M2 account foundation handoff

## Status: implemented foundation; M2 is not accepted or customer-live

- **Implementation commit:** `08b2470eaff37e998e30323e98c2f87d8750baf6`
- **Live build:** `08b2470eaff37e998e30323e98c2f87d8750baf6`
- **Live image:** `sociobotregistry.azurecr.io/sf-mtd-quarterly-ready@sha256:c365807e7e365a3f8f0ee016a5a4b686aa5333bff778a554ed0cdafc62085e98`
- **Current milestone:** M2 implementation only. M1 remains the accepted public workflow.

## What changed

The service now has an account data boundary ready for an authorised Sociobot Entra CIAM public-client registration: OIDC discovery, PKCE, nonce and browser-bound state validation, signed ID-token checks, HTTP-only sessions, business membership, encrypted account-quarter records, explicit idempotent browser-quarter migration, account export/delete, and account-scoped accountant links. Account routes use the server session and never accept a browser workspace ID as account authority.

`/account` and the records screen state the current live truth. No authorised issuer/client configuration is deployed, so sign-in says it is unavailable. Existing M1 browser records, the `demo:` namespace, free downloads, receipt locality, and reviewed handoff remain unchanged. Demo mode never calls account routes or moves data.

The M1 decorative-label finding remains closed: the first desktop and phone screens use direct labels including `Quarter status` and `Downloads and sharing`.

## Verification

From a fresh clone, `npm ci` completed with 60 packages and zero audit vulnerabilities. Every exact command in `.factory/claims.json` ran independently; all 27 claims passed. The final local `npm test` passed 11 frontend unit tests, 22 Rust tests, the deployment contract, the production build, and 59 Playwright tests. Production output is 17.51 kB gzip JavaScript and 5.57 kB gzip CSS. Formatting, Clippy with warnings denied, TypeScript, and `git diff --check` passed.

New outcome tests prove tenant isolation, explicit non-overwriting browser-quarter migration, and account export/delete without touching another account. The no-CIAM browser test proves the account page leaves browser records unchanged and makes no false sign-in promise.

Fresh live desktop (1440 × 900) and phone (390 × 844) checks had no console errors or overflow. Before scrolling, both showed the job **Turn records into a checked quarterly update**, its UK sole-trader/tutor/landlord audience, and **Try it with sample data**. The sample opened to ten Maya Patel Tutoring rows with £260.00 income, £155.83 costs, £104.17 net, the persistent **Demo — sample data, nothing is saved** label, and Reset demo. Reset restored ten rows. Screenshots are `/work/.evidence/m2-live/`.

Live `verify:url` passed for `/demo` and `/account`. The release verifier passed against this build: immutable image, one active/running replica, Azure Files at `/data`, durable workspace round trip, 20/20 concurrent saves, handoff-only HMRC state, safe non-charging fixture, and 40 read / 12 write per-client allowance followed by 429 and `Retry-After`, including OAuth callback.

## Deployment and dependencies

Deployment retained Single revision mode, one min/max replica, the existing product `/data` Azure Files mount, and handoff-only HMRC configuration. It added no provider, identity, or billing secret.

M2 needs an authorised Sociobot Entra CIAM issuer/client registration and an authorised billing-controller lifecycle to exercise sign-in, purchase, restore/paste, verification, revoke, and recovery end to end. The public metadata for the existing £12/month offer is `/work/.evidence/billing-offer.json`; no payment provider was embedded.

M3 remains separate and unshipped: an approved MTD provider, taxpayer consent, and a controlled provider/HMRC acknowledgement are still required. Live health remains `hmrc_integration_configured:false` and `hmrc_integration_mode:not_configured`.

## Run and verify

```sh
npm ci
npm test
npm run build
EXPECTED_BUILD_SHA=08b2470eaff37e998e30323e98c2f87d8750baf6 npm run verify:release
```
