# Quarterly Ready venture plan

**Planning review:** 5 September 2026 UTC
**Planning basis:** source at `511eed4f0fb8c0f4e15bf897a9240606429518a4`, live build `7c840e4853bbcb16270977bdb568271ebd86c746`, and every verification report from [verification.md](verification.md) through [verification-22.md](verification-22.md).

## Planning decision and current status

Quarterly Ready is a UK record-to-quarterly-update product, not a general ledger or tax adviser. The live product is deliberately **handoff-only**. It helps a person capture and review a quarter, then download a CSV or reviewed handoff. It must not be described as filing to HMRC, providing taxpayer consent, offering sign-in, isolating tenants, or completing a paid subscription.

M1.2 closed the final verification-23 deployment finding on 5 September 2026. The already-tested image is now pinned by immutable registry digest while the existing one-replica `/data` mount is preserved. This operational repair did not add or change a product capability.

The accepted milestone is **M1 — records to reviewed handoff**. Its M1.1 repair deployed on 5 September 2026: both real 404 render paths now use direct recovery wording, and the recovery flow is covered in the browser suite. M1 is accepted only as a non-filing core. M2 and M3 are not completed merely because there are local fixtures or conditional code paths for their features.

| Area | Real status | Evidence |
| --- | --- | --- |
| Landing, one-click demo, capture/review, CSV, reviewed handoff, and demo accountant pack | **Verified live for the non-filing workflow.** The sample flow is useful and isolated. | [verification-22.md](verification-22.md), [demo.md](demo.md), [claims.json](claims.json) |
| Accessibility, responsive behaviour, PWA/offline demo, privacy request boundary, rate limits, encrypted record documents, audit chain, single-replica durable deployment | **Verified in the latest evidence.** These are technical safeguards, not proof of customer accounts or tax filing. | [verification-22.md](verification-22.md), [isolation-2026-09-05.md](isolation-2026-09-05.md) |
| Billing | **Partly demonstrated.** The latest verification observed both checkout endpoints returning permitted hosted URLs and verified the server rejects an unlicensed live-link request. No purchase, renewal, entitlement restoration, or paid customer journey was exercised. | [billing.md](billing.md), [verification-22.md](verification-22.md) |
| Accountant links | **Implemented and tested as a conditional service.** Demo uses a fixture; live links require a server-verified subscription and expire after 30 days. This does not establish an account model or tenant isolation. | [verification-22.md](verification-22.md), [claims.json](claims.json) |
| Approved-provider submission and taxpayer consent | **Demonstrated only with local mocks.** The live health response says `hmrc_integration_configured:false` and `hmrc_integration_mode:"not_configured"`; direct submission is hidden. The non-filing QA/sandbox path files no return. | [verification-22.md](verification-22.md), [isolation-2026-09-05.md](isolation-2026-09-05.md) |
| Accounts, sign-in, and tenant isolation | **M2 foundation, not a live customer capability.** Account routes use server sessions and membership checks when CIAM is configured. The live service has no authorised CIAM registration, so it exposes no sign-in, account migration, tenant isolation, or cross-device recovery claim. M1 browser workspaces remain available only as explicit migration sources. | [README.md](../README.md), [storage.ts](../frontend/src/storage.ts), [main.rs](../src/main.rs) |

### Accepted evidence versus release acceptance

The following prior defects are closed by the latest verification: clean-checkout claim harness, TypeScript check, empty-workspace console error, API validation, receipt quota recovery, real-quarter separation, server-side licence gate, checkout endpoint availability, durable one-replica topology, concurrent-save retention, external rate limiting, service-worker offline reload, and genuine HTTP 404 status. The history and proof are retained in [verification-22.md](verification-22.md) under “Earlier finding disposition.”

One external dependency remains outside M1:

1. **Critical M3 dependency:** no live approved HMRC integration, taxpayer consent, or submission. A mock, a safe QA fixture, a provider-shaped payload, and an HMRC sandbox greeting are not evidence of a real filing capability.

The M3 dependency is not permission to claim M3 is built. M1 is formally accepted for the truthful records-to-handoff workflow.

## PRD

### Customer and promise

The customer is a UK sole trader, tutor, or landlord who keeps transactions in a spreadsheet or notes and must prepare Making Tax Digital records each quarter without adopting a full accounting suite.

**Promise:** Turn a UK quarter of income and costs into a user-reviewed record pack and, once an approved provider is genuinely configured, a consented quarterly update.

### Three jobs to nail

1. Capture/import transactions and receipt evidence, then reveal unresolved categories and missing receipts before the deadline.
2. Let the taxpayer check totals and produce a complete, readable accountant pack without losing the free export path.
3. After explicit review and taxpayer consent, submit through approved software and retain a trustworthy result and audit trail.

### Wedge, evidence, and business model

The wedge is a quarterly compliance runway and accountant handoff for one UK regime. It avoids the general-ledger-first experience that makes Xero-like products feel too broad. The researched demand signals are the two 2026 UK discussions about MTD awareness and spreadsheet dependence plus the compliance/hand-off complaint in the Invoice Ninja issue recorded in [brief.json](brief.json).

The planned commercial model is a **£12/month or £99/year Sociobot subscription** for live accountant links and any approved-provider service. CSV export, data safety, accessibility, and the basic record workflow remain free. The production billing controller is external; this repository must never contain payment-provider identifiers or credentials.

### Deliberate limits

- UK-only; no payroll, international tax, lending, or autonomous tax advice.
- No claim that a handoff JSON is an HMRC submission.
- No claim of an account, tenant isolation, recovery, or paid entitlement before M2 is accepted.
- No email, SMS, or other messaging is shipped or required in M1–M3. There is no messaging dependency to represent as implemented.
- No AI feature is planned. It would not make the core compliance job safer, and no financial record is sent to a model.

## Current architecture and data boundaries

### What is deployed now

The product is a Vite/TypeScript frontend served by a Rust 2021 Axum service on `PORT` (default 8080). The backend uses SQLite, an AES-GCM key persisted under `/data`, encrypted JSON record documents, a hash-chained audit log, per-client request limiting, and a health response with the build and HMRC capability state. It keeps the active SQLite file on local disk, atomically persists a snapshot to the product-owned `/data` mount after acknowledged mutations, and is intentionally limited to one replica. The live deployment was verified with that topology; this is not a shared database design.

Current browser/server boundaries are intentionally narrow:

| Data | Current owner and boundary |
| --- | --- |
| Demo records and actions | `demo:` localStorage/IndexedDB namespace. Demo actions do not call workspace, share, billing, consent, or submission APIs. Reset discards it. |
| Real transaction document | Browser localStorage plus an encrypted SQLite document addressed by a browser-held workspace UUID. Each quarter has a separate browser key and UUID. |
| Receipt file bytes | Browser IndexedDB only (`quarterly-ready-receipts-v1`). The server receives the receipt name, not the bytes. This is a locality feature, not device recovery. |
| Accountant pack | CSV/handoff is generated in the browser. A live read-only share is an encrypted server snapshot with a 30-day expiry, subject to server-side subscription verification. |
| Minimal measurement | Same-origin daily page count only; no IP address field, advertising cookies, or third-party analytics were verified. |
| Billing | Explicit, user-triggered request to the Sociobot controller only. The current endpoint check is not a paid-customer proof. |

The M1 workspace UUID is not an identity boundary. It remains only to preserve accepted browser records until a signed-in customer explicitly migrates a quarter. The M2 account API derives account-data ownership from a verified session and membership, never from a client-selected workspace ID. Current live configuration still does not create public account tenancy, recovery, or self-service server deletion because CIAM is not enabled.

### Target architecture by milestone

M2 adds a Sociobot Entra CIAM-backed identity boundary: user subject, business/tenant, membership/role, quarter, encrypted quarter document, audit entry, entitlement, and expiring share. Every read/write/share/consent route is scoped by the verified subject and tenant; client-supplied workspace IDs cease to be authority. Existing browser-only receipt bytes remain browser-only unless a later, explicitly consented encrypted-upload design is separately planned.

M3 retains the reviewed document/audit trail and adds a product-owned approved-provider adapter. Consent state and provider tokens stay encrypted server-side; the browser never receives provider credentials. Submission creates an immutable audit event and stores the provider/HMRC receipt reference, status, and error state. The adapter is disabled and the UI remains handoff-only whenever the live capability check is not affirmative.

## Design system and key screens

The accepted identity is the **mid-century instrument panel** documented in [design.md](design.md): warm paper/panel/brass colours, Georgia plus system sans, an 8-pixel rhythm, labelled controls, a quarter dial, visible status lamps, and a 240 ms dial sweep that becomes instant under reduced motion. The original generated instrument still-life and its provenance are also recorded there.

Key screens are: landing and live preview; demo/real quarter desk; transaction capture/import and receipt recovery; quarter checklist/review; downloads and sharing; authenticated account/business selector in M2; and conditional consent/submission confirmation in M3. All retain one H1, landmarks, route focus announcements, designed focus rings, 44 px controls, error recovery, and the 390 px labelled-slip layout. M1.1 made the 404 screen use direct recovery language without changing this visual system.

## Milestones

### M1 — records to reviewed handoff

**Scope:** landing, `/demo`, `/records`, `/privacy`, `/terms`, manual capture, bank CSV import, browser-local receipt capture, unresolved-work checklist, explicit human review, free CSV and reviewed handoff downloads, and demo accountant pack.

**Status:** **accepted on 5 September 2026 for the non-filing workflow.** M1.1 replaced the two metaphorical 404 strings with direct recovery wording, proved the real 404 and in-app fallback return a visitor home, and re-verified the live handoff-only deployment. It is not an accepted solution to the entire researched MTD-submission job.

**Definition of done:**

- A cold visitor can use `/demo` in one click, reset it, leave it, and complete the sample records-to-handoff flow without an account.
- Demo data is isolated from real browser/server data; receipt bytes remain outside localStorage and server records.
- A real browser workspace can create a UK quarter, validate/import records, resolve checklist failures, persist a reviewed document, and freely download CSV/handoff output.
- Empty, invalid, offline, quota, keyboard, mobile, reduced-motion, legal, privacy, route, and 404 recovery states pass; all public copy truthfully says the product is handoff-only.
- M1.1 completed the direct 404 wording, focused 404 recovery, `verify:url`, Axe integration, link crawl, and full browser regression without broadening a public claim.

**Claims and tests:** Existing claim IDs `demo-isolation`, `demo-access`, `privacy-no-tracking`, `accountant-csv`, `quarter-review`, `free-quarter-persistence`, `csv-import`, `receipt-capture`, `receipt-locality`, `quarter-record-separation`, `hmrc-handoff`, and `offline-browser-copy` are the M1 evidence set. Each exact command lives in [claims.json](claims.json); the latest independent run passed all 24 manifest commands, with the M1 set exercised in the demo/live browser suite. See [verification-22.md](verification-22.md).

**No external dependency blocks M1.** It did not turn on provider, billing, sign-in, or messaging work.

### M2 — authenticated records, tenancy, and paid service

**Scope:** replace browser-UUID authority with accounts and tenant-scoped persistence; preserve the demo; deliver purchase/restore and a server-verified entitlement for live accountant links.

**Status:** **M2 implementation is in progress, but not shipped as a customer capability.** The repository now has the account data model, OIDC/PKCE session boundary, membership checks, explicit idempotent browser-quarter migration, account export/delete, and account-scoped link route. The live service has no authorised Sociobot Entra CIAM issuer/client registration, so `/account` truthfully says sign-in is unavailable and no account/tenant/cross-device claim is public. Existing M1 browser workspaces remain intact for migration and are not an account authority. M2 cannot be accepted until CIAM sign-in and the authorised entitlement/restore journey are exercised end to end.

**Definition of done:**

- A customer can create/sign in to an account through Sociobot Entra CIAM, create a business, and return to only their own quarters across devices.
- Every API route that reads/writes a real record, link, consent, or submission verifies the signed identity and tenant membership. Two users/tenants cannot enumerate, read, mutate, share, or consent each other’s data.
- Migration from the current anonymous browser workspace is explicit, reviewed, idempotent, and does not import demo data. Account data has self-service export and deletion behaviour that matches the privacy page.
- The £12/month and £99/year controller journeys support restore/paste and server-side verification. A controller test path may prove wiring without charging, but M2 acceptance also needs an authorised end-to-end entitlement/restore check; a checkout URL alone is insufficient.
- The service retains the current one-replica durable SQLite safeguards, encrypted records, audit log, rate limits, health/build identity, and no receipt-byte upload. Add tenant-isolation, auth failure, migration, export/delete, billing failure/retry, and account recovery tests.

**Claims and tests to carry forward:** `accountant-link`, `accountant-link-expiry`, `server-licence-gate`, and `paid-tier` need authenticated, observable end-to-end coverage. The implementation adds exact isolated-server checks for `tenant-isolation`, `account-migration`, and `account-export-delete`; they prove the account boundary but do not substitute for authorised CIAM sign-in. Add entitlement restoration only after the controller provides an authorised lifecycle path. The existing server-gate and paid-tier checks are passing evidence only; neither proves a purchased subscription nor a tenant boundary.

**External dependencies:** Sociobot Entra CIAM application/issuer setup and the Sociobot billing-controller registrations/verification service. The billing endpoint availability was observed in verification 22; account identity and paid-customer verification have not been supplied or tested. Future authorized operators provision those integrations; product workers do not request, inspect, or report credentials.

### M3 — accountant collaboration and approved MTD submission

**Scope:** authenticated accountant sharing plus a real, consented approved-provider route from reviewed quarter to recorded submission result.

**Status:** **blocked and not live.** The repository has conditional provider/consent/submission code and passing mock tests. The live capability is explicitly `not_configured`, so none of it counts as shipping M3.

**Definition of done:**

- An authenticated owner can create/revoke an expiring read-only accountant pack for their tenant; the recipient cannot edit it or traverse to another tenant.
- The UI shows a consent action only when the product-owned approved-provider capability is live. The consent start sends no quarter records. The user returns from consent to a fresh review of the same tenant/quarter.
- Submission is disabled until every review condition and the final human confirmation are true. It sends the mapped, validated period update only through the approved provider; it records outcome, provider/HMRC reference, timestamp, and an auditable failure state. It never reports a handoff, sandbox check, or provider timeout as a filing.
- Contract/unit tests continue to use fixtures; an authorised non-production provider flow proves callback, consent, payload, error, and result handling. M3 is accepted only when an authorised controlled live/pilot filing yields a real provider/HMRC acknowledgement for a reviewed quarter. A mock and the current non-filing HMRC sandbox greeting are explicitly insufficient.
- Privacy, retention, deletion/export, user-facing recovery, support escalation, and conservative compliance copy are reviewed for the actual provider data flow before enabling any submission control.

**Claims and tests to keep honest:** `hmrc-consent-no-records`, `conditional-submission`, `hmrc-submission`, and `hmrc-sandbox-no-filing` remain fixture/conditional tests. Their current passing status proves guards and payload logic, not provider availability. Add a separately named live-capability test only after an authorised environment exists; do not make a live-filing claim until it has passed.

**Exact blockers/dependencies:** an approved MTD provider contract and approval reference; operator-provisioned product-owned runtime configuration for the provider and OAuth consent callback; an authorised test/pilot taxpayer consent; and documented acceptance of the provider’s payload/result semantics. None is present in the live service. No production credential should be requested from or exposed to a product worker.

## External dependency ledger

| Dependency | Applies to | Current evidence/status | Required next action |
| --- | --- | --- | --- |
| Product-owned `/data`, immutable image, single active replica | M1/M2/M3 | Verified after the M1.2 deployment in [handoff-m1.md](handoff-m1.md); the verification-23 mutable-tag finding is closed. | Preserve and reverify after runtime changes. |
| Sociobot billing controller and two subscriptions | M2/M3 | Checkout URLs and unlicensed-link rejection observed; purchase/restore not exercised. | Authorised billing verification; do not embed a payment provider. |
| Sociobot Entra CIAM | M2/M3 | No sign-in path or tenant identity exists. | Authorised identity integration, then isolation tests. |
| Approved MTD provider and taxpayer OAuth consent | M3 | Live health is `not_configured`; only mocks/non-filing fixture exist. | Separate authorised integration work and controlled provider verification. |
| HMRC direct access | M3 | Not configured and not claimed. | Only through the approved-provider route; never infer access from sandbox/mock tests. |
| Messaging/email | none in M1–M3 | Not built, marketed, or required by the brief. | Keep out of scope unless a later plan adds an opt-in transactional need. |

## Risks, experiments, and release rule

| Risk/unknown | Retirement experiment |
| --- | --- |
| Anonymous browser UUIDs can be copied, lost, or used as authority. | M2 two-tenant integration tests plus account recovery/export/delete exercise. |
| Browser-only receipts may be unavailable on another device or to an accountant. | Interview/pilot test whether transaction-level receipt names plus CSV meet the handoff need before designing any encrypted upload; do not silently change the locality promise. |
| Provider payload semantics or approval conditions differ from the local mock. | M3 provider contract test and authorised non-production consent flow, followed by a controlled pilot acknowledgment. |
| A checkout redirect is mistaken for a working subscription. | M2 authorised entitlement, revoke, restore, and server-gate test with no direct payment-provider integration. |
| Compliance copy outruns capability. | Every release cross-checks the landing, README, legal pages, [claims.json](claims.json), `/health` capability, and actual provider status. |

**Release rule:** Do not call the product an HMRC filing service or a tenant-isolated subscription SaaS until M2/M3 acceptance criteria have independently passed. Until then, ship only the truthful non-filing records-to-handoff promise.
