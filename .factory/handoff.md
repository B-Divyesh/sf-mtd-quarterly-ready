# Quarterly Ready — isolation cleanup handoff

## Status: complete — scoped configuration repair

The default deployment script no longer reads from a shared Key Vault or binds HMRC secret references. It now has an explicit product-owned `handoff-only` contract, so a release cannot imply that direct HMRC submission is available.

## What changed

- `scripts/deploy-container.sh` defaults to and permits only `DEPLOYMENT_MODE=handoff-only`; a different mode exits before build or deployment work.
- Its Container App payload has an empty secret set and explicitly configures `HMRC_INTEGRATION_MODE=not_configured`.
- `scripts/check-deploy-contract.mjs` enforces the new mode and rejects shared-vault and secret-reference regressions.
- Release verification now asserts the deployed handoff-only capability; README deployment guidance matches the contract.

## How verified

- `bash -n scripts/deploy-container.sh` passed.
- `npm run test:deploy-contract` passed.
- An unsupported deployment mode exited with status 2 before the build step.
- A source-only executable-config grep found no shared-vault or HMRC-secret mechanism in `scripts/deploy-container.sh`.

No deployment was run, no credentials were read, and the existing SQLite `/data` storage configuration was not changed.

## Known gap / next step

Direct HMRC submission is intentionally unavailable. The product supports records, CSV, and accountant handoffs without claiming a filing occurred. Any future filing work needs a separately reviewed, product-owned integration contract; this repair neither obtains nor supplies credentials.

See [the isolation report](isolation-2026-09-05.md) for redacted evidence.
