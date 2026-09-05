# Scoped isolation report — 2026-09-05

## Scope

This repair changed only Quarterly Ready release configuration and its local validation/docs. No deployment, shared-service access, runtime-storage operation, or credential lookup was performed.

## Result

`scripts/deploy-container.sh` now has a product-owned, fail-closed configuration contract:

- `DEPLOYMENT_MODE` defaults to, and accepts only, `handoff-only`.
- The Container App payload declares an empty secret set and explicitly sets the HMRC capability to `not_configured`.
- The script contains no shared-vault endpoint, Key Vault command, Key Vault URL, secret reference, or HMRC secret-name variable.
- A non-handoff mode exits with status 2 before the image build or any Container App change.

The release verifier and package release command now require the observed handoff-only state. The deployment-contract checker rejects a regression to any of the removed out-of-scope configuration mechanisms.

## Redacted verification evidence

All checks were local and inspect source only:

```text
bash -n scripts/deploy-container.sh
result: passed

npm run test:deploy-contract
result: passed — contract reports explicit handoff-only HMRC state and no shared-vault secret references

DEPLOYMENT_MODE=approved bash scripts/deploy-container.sh
result: exited 2 before build; fail-closed mode message confirmed

grep executable deployment configuration for shared-vault and HMRC-secret mechanisms
result: no matches
```

No secret values, secret identifiers, credentials, or external-service settings were read or logged. SQLite `/data` configuration and the existing durable-storage path were not changed.

## Deliberate limitation

Direct HMRC submission is unavailable in this deployment contract. Quarterly Ready remains honest about that state and offers records, CSV exports, and accountant handoffs only.
