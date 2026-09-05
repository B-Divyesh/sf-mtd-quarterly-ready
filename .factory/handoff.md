# Quarterly Ready — M1 build 2 handoff

## Result: PASS

Verification-23's only current-milestone finding is fixed. The live deployment
uses immutable image
`sociobotregistry.azurecr.io/sf-mtd-quarterly-ready@sha256:359061bec80ef1cb2c9339c228f1f5f5cbaddf0c61301cd38163659fcc088585`.
The exact release verifier passes.

No product code changed. Application implementation remains `13380e4`; the
deployed tested image reports `89338a9`. This later handoff is documentation,
not a new product image.

## Proof

- Live health: 200, build `89338a9`, safe QA fixture enabled, HMRC mode
  `not_configured`.
- Live topology: Single mode, min/max 1/1, one running replica, the existing
  `mtd-quarterly-ready-data-v3` Azure Files mount at `/data`, exact digest
  above.
- `/data` preservation: ten pre-deployment workspaces and one encrypted link
  survived deployment and 60 routed reads.
- Release verification: 20/20 concurrent saves, monthly and annual checkout,
  exact 40/12 API allowances, and 429 plus `Retry-After` all passed.
- Clean clone: 24/24 claim commands and `npm test` passed; formatting, Clippy,
  audit, production build, and release build passed.
- Live browser: 54 applicable tests passed with one expected ingress-only
  skip; Axe found no serious/critical issues and `verify:url` passed.
- Fresh desktop/phone sample, downloads, persistent demo label, reset, and
  real-data isolation passed. Lighthouse scored 100/100/100/100.

Detailed milestone evidence and remaining future dependencies are in
[handoff-m1.md](handoff-m1.md). M2 accounts/tenancy and M3 approved-provider
submission are not shipped and were not pulled into M1.
