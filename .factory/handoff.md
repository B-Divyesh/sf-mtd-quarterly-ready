# Quarterly Ready — venture plan handoff

## Status: plan complete; product release remains FAIL

This planner work order made no product-code, deployment, infrastructure, billing, or external-service change. It inspected the researched brief, current implementation, all repository QA reports/evidence, and the live health endpoint, then added the venture plan at [plan.md](plan.md) and the machine-readable status at `/work/.evidence/venture-plan.json`.

The live service still reports build `7c840e4853bbcb16270977bdb568271ebd86c746` with `hmrc_integration_configured:false` and `hmrc_integration_mode:"not_configured"`. The product is correctly handoff-only. It does not provide live HMRC filing, taxpayer consent, sign-in, tenant isolation, or a proven paid-customer lifecycle.

## Current milestone

**M1.1 — repair and re-verify the non-filing core.**

The non-filing records-to-reviewed-handoff workflow is well evidenced, but formal M1 acceptance is pending the small plain-words repair to the 404 page. M2 (accounts, tenancy, persistence ownership, and paid lifecycle) is not a shipped capability. M3 (approved provider, consent, and filing) is blocked on an external product-owned integration and is not shipped.

## What was verified in this planning checkout

```sh
npm ci
npm test
```

Passed: TypeScript typecheck, 11 Vitest tests, 18 Rust tests, deployment-contract check, Vite build, and 54 Playwright tests. The built assets were 15.59 kB gzip JavaScript and 5.33 kB gzip CSS.

Read-only live check on 5 September confirmed the deployed health capability is handoff-only. The latest independent evidence remains [verification-22.md](verification-22.md): 24/24 declared claims passed, 53 live browser tests passed with one inapplicable non-claim skip, and mobile Lighthouse scored 100/100/100/100. The latest independent verdict remains FAIL because no approved direct-submission integration exists and the 404 copy is metaphorical.

## Open work and exact dependencies

1. **Next work order — M1.1:** replace `NO SIGNAL` / `This page is not on the panel` with direct page-not-found recovery copy. Re-run focused 404, link, accessibility, and browser checks. Do not broaden scope.
2. **M2:** introduce real Sociobot Entra CIAM identity and server-enforced tenant ownership, explicit migration/export/delete, and a verified billing lifecycle. Checkout URLs and UUID-addressed encrypted records do not prove these capabilities.
3. **M3 (critical external blocker):** an authorised operator must provide a product-owned approved MTD provider/consent integration. A mock bridge, safe QA fixture, and non-filing HMRC sandbox greeting do not prove or enable filing. Do not request or place credentials in this repository.

The full scope, definitions of done, test/claim separation, data boundaries, risk experiments, and dependency ledger are in [plan.md](plan.md). The pre-existing `graphify-out/` changes were preserved and remain unstaged.
